use futures::TryStreamExt;
use rspotify::{
    model::{FullTrack, PlayableItem, PlaylistId, SearchResult, SearchType, SimplifiedPlaylist},
    prelude::{BaseClient, OAuthClient},
    AuthCodePkceSpotify, Config, Credentials, OAuth,
};
use std::collections::HashSet;

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
