use dashmap::DashMap;
use parking_lot::RwLock;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::model::FullTrack;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;

const QUESTION_TIMEOUT_SECS: u64 = 10;
const NEXT_QUESTION_WAIT_TIME_MS: u128 = 1500;
const SCORE_LIMIT: u64 = 2000;

#[derive(Debug)]
pub struct Room {
    pub update_broadcast: tokio::sync::broadcast::Sender<()>,
    pub game: RwLock<GameState>,
    pub users: DashMap<String, User>,
}

impl Room {
    pub fn new() -> Self {
        let (update_broadcast, _) = tokio::sync::broadcast::channel(10);
        Self {
            update_broadcast,
            game: RwLock::new(GameState::Waiting),
            users: DashMap::new(),
        }
    }

    pub fn users(&self) -> Vec<User> {
        self.users.iter().map(|u| u.value().clone()).collect()
    }

    pub fn on_question_end(&self) {
        let mut game = self.game.write();
        if let GameState::Playing(state) = &mut *game {
            if state.question_state.status != QuestionStatus::Playing {
                return;
            }

            for submission in &mut state.question_state.submissions {
                if let Some(mut user) = self.users.get_mut(&submission.user_id) {
                    let score =
                        state.questions[state.question_state.id].submission_score(submission);
                    submission.score = Some(score);
                    user.score += score;
                }
            }

            state.question_state.end_question();
            let _ = self.update_broadcast.send(()); // ignore broadcast send error
        }
    }

    pub fn on_question_next(&self) {
        let mut game = self.game.write();
        if let GameState::Playing(state) = &mut *game {
            if state.question_state.status != QuestionStatus::Ended {
                return;
            }

            if state.question_state.id == state.questions.len() - 1 {
                *game = GameState::Ended {
                    playlist_id: state.playlist_id.clone(),
                    num_questions: state.questions.len(),
                };
            } else {
                state.question_state.next_question();
            }

            let _ = self.update_broadcast.send(());
        }
    }

    pub fn new_game(&self, playlist_id: String, questions: Vec<Question>) {
        for mut user in self.users.iter_mut() {
            user.score = 0;
        }

        let mut game = self.game.write();
        *game = GameState::Playing(PlayingGameState {
            playlist_id,
            questions,
            question_state: QuestionState::new(),
        });
        let _ = self.update_broadcast.send(());
    }

    pub fn periodic_update(&self) {
        let game = self.game.read();

        if let GameState::Playing(state) = &*game {
            if state.question_state.status == QuestionStatus::Playing {
                // end the current question if time is up
                if state.question_state.timer.elapsed().as_secs() > QUESTION_TIMEOUT_SECS {
                    drop(game);
                    self.on_question_end();
                }
            } else if state.question_state.timer.elapsed().as_millis() > NEXT_QUESTION_WAIT_TIME_MS
            {
                // move to the next question if time is up
                drop(game);
                self.on_question_next();
            }
        }
    }

    pub fn on_user_join(&self, user_id: &str, user_name: &str) {
        match self.users.entry(user_id.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                entry.get_mut().online = true;
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(User::new(user_name.to_string()));
            }
        }
        let _ = self.update_broadcast.send(());
    }

    pub fn on_user_leave(&self, user_id: &str, remove: bool) {
        if remove {
            self.users.remove(user_id);
        } else if let Some(mut user) = self.users.get_mut(user_id) {
            user.online = false;
        }
        let _ = self.update_broadcast.send(());
    }
}

#[derive(Debug)]
pub struct PlayingGameState {
    pub playlist_id: String,
    pub questions: Vec<Question>,
    pub question_state: QuestionState,
}

impl PlayingGameState {
    pub fn current_question(&self) -> &Question {
        &self.questions[self.question_state.id]
    }
}

#[derive(Debug)]
pub enum GameState {
    Waiting,
    Playing(PlayingGameState),
    Ended {
        playlist_id: String,
        num_questions: usize,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum QuestionStatus {
    Playing,
    Ended,
}

#[derive(Debug)]
pub struct QuestionState {
    pub id: usize,
    pub submissions: Vec<UserSubmission>,
    pub timer: Instant,
    pub status: QuestionStatus,
}

impl QuestionState {
    pub fn new() -> Self {
        Self {
            id: 0,
            submissions: Vec::new(),
            timer: Instant::now(),
            status: QuestionStatus::Playing,
        }
    }

    pub fn end_question(&mut self) {
        self.timer = Instant::now();
        self.status = QuestionStatus::Ended;
    }

    pub fn next_question(&mut self) {
        self.id += 1;
        self.submissions.clear();
        self.timer = Instant::now();
        self.status = QuestionStatus::Playing;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub score: u64,
    pub online: bool,
}

impl User {
    pub fn new(name: String) -> Self {
        Self {
            name,
            score: 0,
            online: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubmission {
    pub user_name: String,
    pub user_id: String,
    pub choice: usize,
    pub score: Option<u64>,
    // user submission timestamp in ms w.r.t the start of the question
    pub submitted_at_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub choices: Vec<Choice>,
    pub song_url: String,
    pub score: u64,
    #[serde(skip)]
    pub ans_id: usize,
}

impl Question {
    pub fn submission_score(&self, sub: &UserSubmission) -> u64 {
        if sub.choice == self.ans_id {
            // the score is reduced linearly based on the time taken to submit
            // and is reduced closer to (score / 2) if the user submits near the timeout
            self.score
                - ((self.score / 2) * (sub.submitted_at_ms as u64) / 1000 / QUESTION_TIMEOUT_SECS)
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub artists: String,
    #[serde(skip)]
    pub preview_url: String,
    #[serde(skip)]
    pub weight: i64,
}

impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

impl Eq for Choice {}

impl PartialOrd for Choice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.weight).cmp(&(other.weight)))
    }
}

impl Ord for Choice {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.weight).cmp(&(other.weight))
    }
}

pub fn gen_questions(tracks: Vec<FullTrack>, num_questions: usize) -> Vec<Question> {
    let mut questions: Vec<Question> = Vec::new();
    let mut heap: BinaryHeap<Choice> = BinaryHeap::new();
    let mut rng = thread_rng();
    let mut seen_names = HashSet::new();

    let mut score: u64 = 500;
    for track in tracks {
        if seen_names.contains(&track.name) {
            continue;
        }
        seen_names.insert(track.name.clone());

        let preview_url = if let Some(url) = track.preview_url {
            url
        } else {
            continue;
        };
        heap.push(Choice {
            name: track.name,
            artists: track
                .artists
                .into_iter()
                .map(|a| a.name)
                .fold(String::new(), |s, a| s + &a + ", "),
            preview_url,
            weight: track.popularity as i64,
        });
    }

    for _ in 0..num_questions {
        let mut top_choices: Vec<Choice> = Vec::new();

        for _ in 0..4 {
            if let Some(choice) = heap.pop() {
                top_choices.push(choice);
            }
        }

        top_choices.shuffle(&mut rng);

        let ans_id = rng.gen_range(0..4);
        let question = Question {
            choices: top_choices.clone(),
            song_url: top_choices[ans_id].preview_url.clone(),
            ans_id,
            score,
        };
        if score + 100 <= SCORE_LIMIT {
            score += 100;
        }
        questions.push(question);

        for (index, mut choice) in top_choices.into_iter().enumerate() {
            if index != ans_id {
                choice.weight -= rng.gen_range(5..10);
            } else {
                choice.weight -= 18;
            }
            heap.push(choice);
        }
    }

    questions
}

pub fn gen_id(len: usize) -> String {
    thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
