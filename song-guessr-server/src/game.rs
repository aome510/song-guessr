use crate::model::{Choice, Question};
use dashmap::DashMap;
use parking_lot::Mutex;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::model::FullTrack;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashSet};
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
}

impl User {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug)]
pub struct GameState {
    pub update: broadcast::Sender<()>,
    pub questions: Vec<Question>,
    pub current_question_id: Mutex<usize>,
    pub users: DashMap<String, User>,
}

impl GameState {
    pub fn new(questions: Vec<Question>) -> Self {
        let (update, _) = broadcast::channel(10);
        Self {
            update,
            questions,
            current_question_id: Mutex::new(0),
            users: DashMap::new(),
        }
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
        heap.push(track.into());
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

pub fn gen_game_id() -> String {
    thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect()
}
