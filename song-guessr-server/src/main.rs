use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::Context;
use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::get,
    Router,
};
use rspotify::{model::PlayableItem, prelude::BaseClient, AuthCodeSpotify};
use tower_http::cors::CorsLayer;

mod auth;

struct AppState {
    client: AuthCodeSpotify,
}

type SharedState = Arc<RwLock<AppState>>;

async fn _get(state: SharedState) -> anyhow::Result<String> {
    let state = state.read().await;
    let id = rspotify::model::PlaylistId::from_id("37i9dQZF1DXbSWYCNwaARB")?;
    let playlist = state.client.playlist(id, None, None).await?;
    let id = rand::random::<usize>() % playlist.tracks.items.len();
    match &playlist.tracks.items[id].track {
        Some(PlayableItem::Track(track)) => {
            Ok(track.preview_url.clone().context("no preview url")?)
        }
        _ => Err(anyhow::anyhow!("invalid track")),
    }
}

async fn get_api(State(state): State<SharedState>) -> Result<String, String> {
    _get(state).await.map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache =
        librespot_core::cache::Cache::new(Some(std::path::Path::new("/tmp")), None, None, None)?;
    let token = auth::get_token(cache).await?;
    let client = rspotify::AuthCodeSpotify::from_token(token);
    let state = Arc::new(RwLock::new(AppState { client }));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);
    let app = Router::new()
        .route("/get", get(get_api))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
