use dashmap::DashMap;
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;

use crate::{client, game, model::Question};

struct AppState {
    client: client::Client,
    game: DashMap<String, game::GameState>,
}

// TODO: properly classify the error
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

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewGameRequest {
    playlist_id: String,
    num_questions: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewGameResponse {
    game_id: String,
}

async fn new_game(
    State(state): State<Arc<AppState>>,
    Json(request): Json<NewGameRequest>,
) -> Result<Json<NewGameResponse>, AppError> {
    let NewGameRequest {
        playlist_id,
        num_questions,
    } = request;

    let num_questions = num_questions.unwrap_or(15);
    let tracks = state.client.playlist_tracks(playlist_id).await?;
    let questions = game::gen_questions(tracks, num_questions);

    let game = game::GameState::new(questions);
    let game_id = game::gen_game_id();
    state.game.insert(game_id.clone(), game);

    Ok(Json(NewGameResponse { game_id }))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct GameResponse {
    id: usize,
    question: Question,
}

async fn game(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<GameResponse>, AppError> {
    match state.game.get(&id) {
        Some(game) => {
            let response = GameResponse {
                id: game.current_question_id,
                question: game.questions[game.current_question_id].clone(),
            };
            Ok(Json(response))
        }
        None => Err(anyhow::anyhow!("game {id} not found").into()),
    }
}

async fn search_playlist(
    Path(query): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SimplifiedPlaylist>>, AppError> {
    Ok(state.client.search_playlist(query).await.map(Json)?)
}

pub fn new_app(client: client::Client) -> Router {
    let state = Arc::new(AppState {
        client,
        game: DashMap::new(),
    });

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/game", post(new_game))
        .route("/game/:id", get(game))
        .route("/search/:query", get(search_playlist))
        .layer(cors)
        .with_state(state)
}
