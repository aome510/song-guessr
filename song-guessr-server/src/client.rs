use crate::model::{Choice, Question};
use futures::TryStreamExt;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rspotify::{
    model::{FullTrack, PlayableItem, PlaylistId, SearchResult, SearchType, SimplifiedPlaylist},
    prelude::{BaseClient, OAuthClient},
    AuthCodePkceSpotify, Config, Credentials, OAuth,
};
use std::collections::{BinaryHeap, HashSet};

const REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
const SPOTIFY_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
// based on https://github.com/librespot-org/librespot/blob/f96f36c064795011f9fee912291eecb1aa46fff6/src/main.rs#L173
const OAUTH_SCOPES: &[&str] = &[
    "app-remote-control",
    "playlist-modify",
    "playlist-modify-private",
    "playlist-modify-public",
    "playlist-read",
    "playlist-read-collaborative",
    "playlist-read-private",
    "streaming",
    "ugc-image-upload",
    "user-follow-modify",
    "user-follow-read",
    "user-library-modify",
    "user-library-read",
    "user-modify",
    "user-modify-playback-state",
    "user-modify-private",
    "user-personalized",
    "user-read-birthdate",
    "user-read-currently-playing",
    "user-read-email",
    "user-read-play-history",
    "user-read-playback-position",
    "user-read-playback-state",
    "user-read-private",
    "user-read-recently-played",
    "user-top-read",
];

pub struct Client {
    spotify: AuthCodePkceSpotify,
}

impl Client {
    pub fn new() -> Self {
        let oauth = OAuth {
            redirect_uri: REDIRECT_URI.to_string(),
            scopes: HashSet::from_iter(OAUTH_SCOPES.iter().map(|s| s.to_string())),
            ..Default::default()
        };
        let creds = Credentials::new_pkce(SPOTIFY_CLIENT_ID);
        let config = Config {
            token_cached: true,
            cache_path: std::path::PathBuf::from("/tmp/spotify_token_cache.json"),
            ..Default::default()
        };
        Self {
            spotify: AuthCodePkceSpotify::with_config(creds, oauth, config),
        }
    }

    pub async fn get_token(&mut self) -> anyhow::Result<()> {
        let url = self.spotify.get_authorize_url(None)?;
        self.spotify.prompt_for_token(&url).await?;
        Ok(())
    }

    pub async fn search_playlist(&self, query: String) -> anyhow::Result<Vec<SimplifiedPlaylist>> {
        let result = self
            .spotify
            .search(&query, SearchType::Playlist, None, None, None, None)
            .await?;
        match result {
            SearchResult::Playlists(page) => Ok(page.items),
            _ => anyhow::bail!("invalid search result"),
        }
    }

    pub async fn generate_questions(
        &self,
        num_questions: usize,
        playlist_id: String,
    ) -> anyhow::Result<Vec<Question>> {
        let playlist_id = PlaylistId::from_id(playlist_id)?;
        let stream = self.spotify.playlist_items(playlist_id, None, None);

        let mut tracks: Vec<FullTrack> = Vec::new();
        futures::pin_mut!(stream);
        while let Some(item) = stream.try_next().await? {
            let track = item.track.unwrap();
            if let PlayableItem::Track(track) = track {
                tracks.push(track);
            }
        }
        Ok(get_questions(tracks, num_questions))
    }
}

fn get_questions(tracks: Vec<FullTrack>, n_questions: usize) -> Vec<Question> {
    let mut questions: Vec<Question> = Vec::new();
    let mut heap: BinaryHeap<Choice> = BinaryHeap::new();
    let mut rng = thread_rng();

    for track in tracks {
        heap.push(track.into());
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
