use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use tower_http::cors::CorsLayer;

use crate::{client, model};

#[derive(Clone)]
struct AppState {
    client: Arc<client::Client>,
}

struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            // TODO: the error message should be hidden in production
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct QuestionQuery {
    num_questions: Option<usize>,
}

async fn questions(
    Path(playlist_id): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<QuestionQuery>,
) -> Result<Json<Vec<model::Question>>, AppError> {
    let num_questions = query.num_questions.unwrap_or(15);
    Ok(state
        .client
        .generate_questions(num_questions, playlist_id)
        .await
        .map(Json)?)
}

async fn playlist_search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<SimplifiedPlaylist>>, AppError> {
    Ok(state.client.search_playlist(query).await.map(Json)?)
}

pub fn new_app(client: client::Client) -> Router {
    let state = AppState {
        client: Arc::new(client),
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/questions/:id", get(questions))
        .route("/search/:query", get(playlist_search))
        .layer(cors)
        .with_state(state)
}
