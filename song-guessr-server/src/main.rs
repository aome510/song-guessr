use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::get,
    Json, Router,
};
use tower_http::cors::CorsLayer;

mod client;
mod model;

struct AppState {
    client: client::Client,
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
    let num_questions = query.num_questions.unwrap_or(15);
    let state = state.read().await;
    state
        .client
        .generate_questions(num_questions, playlist_id)
        .await
        .map_err(|e| e.to_string())
        .map(Json)
}

async fn playlist_search(
    Path(query): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<Vec<SimplifiedPlaylist>>, String> {
    let state = state.read().await;
    state
        .client
        .search_playlist(query)
        .await
        .map_err(|e| e.to_string())
        .map(Json)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = client::Client::new();
    client.get_token().await?;
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
