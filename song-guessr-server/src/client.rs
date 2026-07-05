use futures::TryStreamExt;
use rspotify::{
    model::{FullTrack, PlayableItem, PlaylistId, SearchResult, SearchType, SimplifiedPlaylist},
    prelude::{BaseClient, OAuthClient},
    AuthCodePkceSpotify, Config, Credentials, OAuth,
};
use std::collections::HashSet;

const REDIRECT_URI: &str = "http://127.0.0.1:8989/login";
const NCSPOT_CLIENT_ID: &str = "d420a117a32841c2b3474932e49fb54b";
// based on https://developer.spotify.com/documentation/web-api/concepts/scopes#list-of-scopes
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
}

impl Client {
    pub fn new() -> Self {
        let oauth = OAuth {
            redirect_uri: REDIRECT_URI.to_string(),
            scopes: HashSet::from_iter(OAUTH_SCOPES.iter().map(|s| s.to_string())),
            ..Default::default()
        };
        let creds = Credentials::new_pkce(NCSPOT_CLIENT_ID);
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
        Ok(tracks)
    }
}
