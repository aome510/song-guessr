use dashmap::DashMap;
use parking_lot::RwLock;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::model::FullTrack;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::time::Instant;

#[derive(Debug)]
pub struct Room {
    pub owner_id: String,
    pub update_broadcast: tokio::sync::broadcast::Sender<()>,
    pub game: RwLock<GameState>,
    pub users: DashMap<String, User>,
}

impl Room {
    pub fn new(owner_id: String) -> Self {
        let (update_broadcast, _) = tokio::sync::broadcast::channel(10);
        Self {
            owner_id,
            update_broadcast,
            game: RwLock::new(GameState::Waiting),
            users: DashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum GameState {
    Waiting,
    Playing {
        questions: Vec<Question>,
        current_question: QuestionState,
    },
    Ended,
}

#[derive(Debug)]
pub struct QuestionState {
    pub id: usize,
    pub submissions: Vec<UserSubmission>,
    pub timer: Instant,
}

impl QuestionState {
    pub fn new() -> Self {
        Self {
            id: 0,
            submissions: Vec::new(),
            timer: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub score: u64,
}

impl User {
    pub fn new(name: String) -> Self {
        Self { name, score: 0 }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSubmission {
    pub user_id: String,
    pub choice: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub choices: Vec<Choice>,
    pub song_url: String,
    #[serde(skip)]
    pub ans_id: usize,
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
        };
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
