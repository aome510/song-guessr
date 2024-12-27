use std::collections::HashSet;

use librespot_core::{authentication::Credentials, cache::Cache, Session, SessionConfig};

pub const SPOTIFY_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";
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

pub async fn get_token(cache: Cache) -> anyhow::Result<rspotify::Token> {
    let creds = match cache.credentials() {
        None => librespot_oauth::get_access_token(
            SPOTIFY_CLIENT_ID,
            "http://127.0.0.1:8989/login",
            OAUTH_SCOPES.to_vec(),
        )
        .map(|t| Credentials::with_access_token(t.access_token))?,
        Some(creds) => creds,
    };

    let session = Session::new(SessionConfig::default(), Some(cache));
    session.connect(creds, true).await?;
    let token = session
        .token_provider()
        .get_token(&OAUTH_SCOPES.join(","))
        .await?;

    let expires_in = chrono::Duration::from_std(token.expires_in)?;
    let expires_at = chrono::Utc::now() + expires_in;
    let scopes = HashSet::from_iter(OAUTH_SCOPES.iter().map(|s| s.to_string()));

    Ok(rspotify::Token {
        access_token: token.access_token,
        expires_in,
        expires_at: Some(expires_at),
        scopes,
        refresh_token: None,
    })
}
