use crate::choice;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::model::track;
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;

#[derive(Serialize, Deserialize)]
pub struct Question {
    pub choices: Vec<choice::Choice>,
    pub ans_id: usize,
}

pub async fn get_questions(tracks: Vec<track::FullTrack>) -> anyhow::Result<Vec<Question>> {
    let mut questions: Vec<Question> = Vec::new();
    let mut heap: BinaryHeap<choice::Choice> = BinaryHeap::new();

    for track in tracks {
        let choice = choice::get_choice(track).await?;
        heap.push(choice);
    }

    // Iterate 15 times
    for _ in 0..15 {
        let mut top_choices: Vec<choice::Choice> = Vec::new();

        // Pop the first 4 elements from the heap
        for _ in 0..4 {
            if let Some(choice) = heap.pop() {
                top_choices.push(choice);
            }
        }

        // Shuffle the 4 elements
        let mut rng = thread_rng();
        top_choices.shuffle(&mut rng);

        // Create a Question instance and push it into the questions vector
        let ans_id = rng.gen_range(0..4);
        let question = Question {
            choices: top_choices.clone(),
            ans_id,
        };
        questions.push(question);

        // Push the shuffled elements back into the heap
        for (index, choice) in top_choices.iter().enumerate() {
            let mut new_choice = choice.clone();
            if index != ans_id {
                new_choice.weight -= rng.gen_range(5..10);
            } else {
                new_choice.weight -= 18;
            }
            heap.push(new_choice);
        }
    }

    Ok(questions)
}
