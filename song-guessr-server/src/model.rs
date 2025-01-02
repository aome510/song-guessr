use rspotify::model::FullTrack;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub name: String,
    pub artist: String,
    #[allow(dead_code)]
    pub preview_url: String,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl From<FullTrack> for Choice {
    fn from(track: FullTrack) -> Self {
        Self {
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
}
