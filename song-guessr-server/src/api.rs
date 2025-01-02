use dashmap::DashMap;
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{client, game, model};

struct AppState {
    client: client::Client,
    game: DashMap<String, Arc<game::GameState>>,
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
    state.game.insert(game_id.clone(), Arc::new(game));

    Ok(Json(NewGameResponse { game_id }))
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    GetCurrentQuestion,
    NextQuestion,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    Question {
        question: model::Question,
        id: usize,
    },
    GameEnded,
}

async fn get_game_ws(
    Path(id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    if let Some(game) = state.game.get(&id) {
        let game = game.clone();
        ws.on_upgrade(move |mut socket| async move {
            if let Err(err) = handle_socket(&mut socket, &game).await {
                // TODO: use logging crate
                eprintln!("Error handling WebSocket: {:?}", err);
            }
        })
    } else {
        (StatusCode::NOT_FOUND, "Game not found").into_response()
    }
}

async fn handle_socket(socket: &mut WebSocket, game: &game::GameState) -> anyhow::Result<()> {
    while let Some(msg) = socket.recv().await {
        if let Message::Text(data) = msg? {
            let msg: WsClientMessage = serde_json::from_str(&data)?;
            match msg {
                WsClientMessage::GetCurrentQuestion => {
                    let current_question_id = *game.current_question_id.lock();
                    match game.questions.get(current_question_id) {
                        Some(question) => {
                            let msg = WsServerMessage::Question {
                                question: question.clone(),
                                id: current_question_id,
                            };
                            let data = serde_json::to_string(&msg)?;
                            socket.send(Message::Text(data)).await?;
                        }
                        None => {
                            socket
                                .send(Message::Text(serde_json::to_string(
                                    &WsServerMessage::GameEnded,
                                )?))
                                .await?;
                        }
                    }
                }
                WsClientMessage::NextQuestion => {
                    let mut current_question_id = game.current_question_id.lock();
                    *current_question_id += 1;
                }
            }
        }
    }
    Ok(())
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

    Router::new()
        .route("/game", post(new_game))
        .route("/game/:id", get(get_game_ws))
        .route("/search/:query", get(search_playlist))
        // .layer(cors)
        .with_state(state)
}
