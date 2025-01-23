use dashmap::DashMap;
use parking_lot::RwLock;
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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

            let fastest_user = state
                .question_state
                .submissions
                .iter()
                .filter(|sub| sub.choice == state.questions[state.question_state.id].ans_id)
                .min_by_key(|sub| sub.submitted_at_ms)
                .map(|sub| sub.user_id.clone());

            for submission in &mut state.question_state.submissions {
                if let Some(mut user) = self.users.get_mut(&submission.user_id) {
                    let is_fastest = fastest_user
                        .as_ref()
                        .map(|id| submission.user_id.eq(id))
                        .unwrap_or(false);
                    let score = state.questions[state.question_state.id]
                        .submission_score(submission, is_fastest);
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
                    question_types: state.question_types.clone(),
                    num_questions: state.questions.len(),
                };
            } else {
                state.question_state.next_question();
            }

            let _ = self.update_broadcast.send(());
        }
    }

    pub fn new_game(
        &self,
        playlist_id: String,
        question_types: Vec<QuestionType>,
        questions: Vec<Question>,
    ) {
        self.users.retain(|_, u| u.online);
        for mut user in self.users.iter_mut() {
            user.score = 0;
        }

        let mut game = self.game.write();
        *game = GameState::Playing(PlayingGameState {
            playlist_id,
            question_types,
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
    pub question_types: Vec<QuestionType>,
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
        question_types: Vec<QuestionType>,
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
    pub question_type: QuestionType,
    pub choices: Vec<String>,
    pub song_url: String,
    pub score: u64,
    pub bonus: u64,
    #[serde(skip)]
    pub ans_id: usize,
}

impl Question {
    pub fn submission_score(&self, sub: &UserSubmission, is_fastest: bool) -> u64 {
        if sub.choice == self.ans_id {
            // the score is reduced linearly based on the time taken to submit
            // and is reduced closer to (score / 2) if the user submits near the timeout
            self.score
                - ((self.score / 2) * (sub.submitted_at_ms as u64) / 1000 / QUESTION_TIMEOUT_SECS)
                + if is_fastest { self.bonus } else { 0 }
        } else {
            0
        }
    }
}

struct Choice<'a> {
    value: &'a str,
    preview_url: &'a str,
    index: usize,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum QuestionType {
    Song,
    Album,
    Artist,
}

impl QuestionType {
    /// Generate a question choice based on the type
    fn gen_choice(self, track: &Track) -> &str {
        match self {
            Self::Song => &track.name,
            Self::Album => &track.album,
            Self::Artist => &track.artists,
        }
    }
}

struct Track {
    name: String,
    album: String,
    artists: String,
    preview_url: String,
    weight: i32,
}

pub fn gen_questions(
    seed_tracks: Vec<rspotify::model::FullTrack>,
    num_questions: usize,
    mut question_types: Vec<QuestionType>,
) -> Vec<Question> {
    let mut rng = thread_rng();

    let mut tracks = Vec::new();
    let mut seen_urls = HashSet::new();

    // process seed tracks
    for track in seed_tracks {
        let preview_url = if let Some(url) = track.preview_url {
            url
        } else {
            continue;
        };

        if seen_urls.contains(&preview_url) {
            continue;
        }
        seen_urls.insert(preview_url.clone());

        tracks.push(Track {
            name: track.name,
            artists: track
                .artists
                .into_iter()
                .map(|a| a.name)
                .fold(String::new(), |acc, a| {
                    if acc.is_empty() {
                        a
                    } else {
                        acc + ", " + &a
                    }
                }),
            album: track.album.name,
            preview_url,
            weight: track.popularity as i32,
        });
    }

    let mut score: u64 = 500;
    let mut questions: Vec<Question> = Vec::new();
    let mut seen_urls = HashMap::new();

    for i in 0..num_questions {
        let mut choices = Vec::<Choice>::new();

        // randomly pick a type for current question
        question_types.shuffle(&mut rng);
        let question_type = question_types[0];

        // pick question choices from the seed tracks based on the track's weight
        // and the current question type
        tracks.sort_by_key(|t| -t.weight);
        for _ in 0..4 {
            for (index, track) in tracks.iter().enumerate() {
                let choice = question_type.gen_choice(track);
                if choices.iter().any(|c| c.value == choice) {
                    continue;
                }
                choices.push(Choice {
                    value: choice,
                    preview_url: &track.preview_url,
                    index,
                });
                break;
            }
        }

        choices.shuffle(&mut rng);

        // generate the answer from the choices
        let mut ans_id = rng.gen_range(0..4);
        // ensure that the same song is not repeated within 10 questions
        while seen_urls
            .get(choices[ans_id].preview_url)
            .map(|j| (i - j) < 10)
            .unwrap_or(false)
        {
            ans_id = rng.gen_range(0..4);
        }
        let song_url = choices[ans_id].preview_url.to_string();
        seen_urls.insert(song_url.clone(), i);

        // construct question for the current round
        let question = Question {
            question_type,
            choices: choices.iter().map(|c| c.value.to_string()).collect(),
            song_url,
            ans_id,
            score,
            bonus: score / 5,
        };
        if score + 100 <= SCORE_LIMIT {
            score += 100;
        }
        questions.push(question);

        // update weight for tracks that are selected as choices
        let penalties = choices
            .into_iter()
            .enumerate()
            .map(|(index, choice)| {
                let penalty = if index != ans_id {
                    rng.gen_range(7..=10)
                } else {
                    20
                };
                (penalty, choice.index)
            })
            .collect::<Vec<_>>();
        for (penalty, index) in penalties {
            tracks[index].weight -= penalty;
        }
    }

    questions
}
