use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::get,
    Json, Router,
};
use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use rspotify::{
    model::{FullTrack, PlayableItem, SearchType, SimplifiedPlaylist},
    prelude::BaseClient,
    AuthCodeSpotify,
};
use tower_http::cors::CorsLayer;

mod auth;
mod model;

struct AppState {
    client: AuthCodeSpotify,
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(Clone, Deserialize, Serialize)]
struct QuestionQuery {
    num_questions: Option<usize>,
}

async fn questions(
    Path(playlist_id): Path<String>,
    State(state): State<SharedState>,
    Query(query): Query<QuestionQuery>,
) -> Result<Json<Vec<model::Question>>, String> {
    async fn api(
        playlist_id: String,
        n_questions: usize,
        state: &AppState,
    ) -> anyhow::Result<Vec<model::Question>> {
        let playlist_id = rspotify::model::PlaylistId::from_id(playlist_id)?;
        let stream = state.client.playlist_items(playlist_id, None, None);

        let mut tracks: Vec<FullTrack> = Vec::new();
        pin_mut!(stream);
        while let Some(item) = stream.try_next().await? {
            let track = item.track.unwrap();
            if let PlayableItem::Track(track) = track {
                tracks.push(track);
            }
        }
        Ok(model::get_questions(tracks, n_questions))
    }

    let state = state.read().await;
    let n_questions = query.num_questions.unwrap_or(15);
    api(playlist_id, n_questions, &state)
        .await
        .map_err(|e| e.to_string())
        .map(Json)
}

async fn playlist_search(
    Path(query): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<Vec<SimplifiedPlaylist>>, String> {
    async fn api(query: String, state: &AppState) -> anyhow::Result<Vec<SimplifiedPlaylist>> {
        let result = state
            .client
            .search(&query, SearchType::Playlist, None, None, None, None)
            .await?;
        match result {
            rspotify::model::SearchResult::Playlists(page) => Ok(page.items),
            _ => Err(anyhow::anyhow!("invalid search result")),
        }
    }

    let state = state.read().await;
    api(query, &state)
        .await
        .map_err(|e| e.to_string())
        .map(Json)
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
        .route("/questions/:id", get(questions))
        .route("/search/:query", get(playlist_search))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
