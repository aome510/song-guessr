use futures::TryStreamExt;
use rspotify::{
    model::{FullTrack, PlayableItem, PlaylistId, SearchResult, SearchType, SimplifiedPlaylist},
    prelude::{BaseClient, OAuthClient},
    AuthCodePkceSpotify, Config, Credentials, OAuth,
};
use serde::Deserialize;
use std::collections::HashSet;

const TRACK_LIMIT: usize = 100;
const REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
const SPOTIFY_CLIENT_ID: &str = "282cce38fbf041ab8325d63464202d6d";
// based on https://github.com/librespot-org/librespot/blob/f96f36c064795011f9fee912291eecb1aa46fff6/src/main.rs#L173
pub const OAUTH_SCOPES: &[&str] = &[
    // Spotify Connect
    "user-read-playback-state",
    "user-modify-playback-state",
    "user-read-currently-playing",
    // Playback
    "app-remote-control",
    "streaming",
    // Playlists
    "playlist-read-private",
    "playlist-read-collaborative",
    "playlist-modify-private",
    "playlist-modify-public",
    // Follow
    "user-follow-modify",
    "user-follow-read",
    // Listening History
    "user-read-playback-position",
    "user-top-read",
    "user-read-recently-played",
    // Library
    "user-library-modify",
    "user-library-read",
];

pub struct Client {
    spotify: AuthCodePkceSpotify,
    deezer: DeezerClient,
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
            deezer: DeezerClient::new(),
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

    pub async fn playlist_tracks(&self, playlist_id: &str) -> anyhow::Result<Vec<FullTrack>> {
        let playlist_id = PlaylistId::from_id(playlist_id)?;
        let stream = self.spotify.playlist_items(playlist_id, None, None);

        let mut tracks: Vec<FullTrack> = Vec::new();
        futures::pin_mut!(stream);
        while let Some(item) = stream.try_next().await? {
            if let Some(PlayableItem::Track(track)) = item.track {
                tracks.push(track);
            }
        }
        // only keep the top TRACK_LIMIT most popular tracks
        tracks.sort_by_key(|t| t.popularity);
        tracks.reverse();
        tracks = tracks.into_iter().take(TRACK_LIMIT).collect();
        let tracks = futures::future::join_all(tracks.into_iter().map(|mut track| async {
            track.preview_url = self.deezer.search_track_preview(&track).await;
            track
        }))
        .await;
        let tracks: Vec<_> = tracks.into_iter().collect();
        Ok(tracks)
    }
}

// Deezer API integration
struct DeezerClient {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct DeezerSearchResponse {
    data: Vec<DeezerTrack>,
}

#[derive(Debug, Deserialize)]
struct DeezerTrack {
    preview: Option<String>,
}

impl DeezerClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn search_track_preview(&self, track: &FullTrack) -> Option<String> {
        // Build search query with track name and artist
        let artist_name = track.artists.first()?.name.clone();
        let track_name = track.name.clone();
        let query = format!("{} {}", artist_name, track_name);

        // Search Deezer API
        let url = format!(
            "https://api.deezer.com/search?q={}",
            urlencoding::encode(&query)
        );

        let response = self.client.get(&url).send().await.ok()?;
        let search_result: DeezerSearchResponse = response.json().await.ok()?;

        // Return the first preview URL found
        search_result
            .data
            .into_iter()
            .find_map(|track| track.preview)
    }
}

#[cfg(test)]
mod tests {
    use rspotify::model::Id;

    use super::*;

    #[tokio::test]
    async fn test_get_playlist_tracks() {
        let mut client = Client::new();
        client.get_token().await.unwrap();
        let playlists = client.search_playlist("bts".to_string()).await.unwrap();
        assert!(!playlists.is_empty(), "Should find at least one playlist");
        let playlist_id = playlists[0].id.id();
        let tracks = client.playlist_tracks(playlist_id).await.unwrap();
        assert!(!tracks.is_empty(), "Playlist should have tracks");
        for track in tracks {
            assert!(
                track.preview_url.is_some(),
                "Track should have a preview URL"
            );
        }
    }
}
