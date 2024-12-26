use rspotify::model::track::FullTrack;
use std::cmp::Ordering;

#[derive(Clone)]
pub struct Choice {
    pub name: String,
    pub artist: String,
    #[allow(dead_code)]
    pub preview_url: String,
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

pub async fn get_choice(track: FullTrack) -> anyhow::Result<Choice> {
    // track name
    let name = track.name;

    // artist names
    let mut artist = String::new();
    for arts in track.artists {
        artist.push_str(&arts.name);
        artist.push_str(", ");
    }
    artist.truncate(artist.len() - 2);

    // preview url
    let preview_url = track.preview_url.unwrap();

    // weight
    let weight = track.popularity as i64;

    Ok(Choice {
        name,
        artist,
        preview_url,
        weight,
    })
}
