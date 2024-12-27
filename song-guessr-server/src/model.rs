use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::model::FullTrack;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub artist: String,
    #[allow(dead_code)]
    pub preview_url: String,
    pub weight: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub choices: Vec<Choice>,
    pub ans_id: usize,
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

fn get_choice(track: FullTrack) -> Choice {
    Choice {
        name: track.name,
        artist: track
            .artists
            .into_iter()
            .map(|a| a.name)
            .fold(String::new(), |s, a| s + &a + ", "),
        preview_url: track.preview_url.unwrap(),
        weight: track.popularity as i64,
    }
}

pub fn get_questions(tracks: Vec<FullTrack>, n_questions: usize) -> Vec<Question> {
    let mut questions: Vec<Question> = Vec::new();
    let mut heap: BinaryHeap<Choice> = BinaryHeap::new();
    let mut rng = thread_rng();

    for track in tracks {
        heap.push(get_choice(track));
    }

    for _ in 0..n_questions {
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
