use dashmap::DashMap;
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::{
    mem,
    sync::Arc,
    time::{Duration, Instant},
};

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

use crate::{client, game};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    UserSubmitted(game::UserSubmission),
    UserJoined { name: String, id: String },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    GameState {
        question: game::Question,
        question_id: usize,
        users: Vec<game::User>,
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

async fn send_game_state_update(
    socket: &mut WebSocket,
    game: &game::GameState,
) -> anyhow::Result<()> {
    let current_question_id = game.current_question.lock().id;
    match game.questions.get(current_question_id) {
        Some(question) => {
            let users = game.users.iter().map(|u| u.value().clone()).collect();
            let msg = WsServerMessage::GameState {
                question: question.clone(),
                question_id: current_question_id,
                users,
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
    Ok(())
}

fn handle_question_end(
    current_question: &mut parking_lot::MutexGuard<game::QuestionState>,
    game: &game::GameState,
) {
    let submissions = mem::take(&mut current_question.submissions);
    for submission in submissions {
        if let Some(mut user) = game.users.get_mut(&submission.user_id) {
            if submission.choice == game.questions[current_question.id].ans_id {
                user.score += 1;
            }
        }
    }

    current_question.id += 1;
    current_question.timer = Instant::now();

    let _ = game.update.send(()); // ignore broadcast send error
}

async fn handle_client_msg(msg: WsClientMessage, game: &game::GameState) -> anyhow::Result<()> {
    match msg {
        WsClientMessage::UserSubmitted(submission) => {
            let mut current_question = game.current_question.lock();
            current_question.submissions.push(submission);

            // end the current question if all users have submitted
            if current_question.submissions.len() == game.users.len() {
                handle_question_end(&mut current_question, game);
            }
        }
        WsClientMessage::UserJoined { name, id } => {
            game.users.entry(id).or_insert(game::User::new(name));
            let _ = game.update.send(()); // ignore broadcast send error
        }
    }
    Ok(())
}

async fn handle_socket(socket: &mut WebSocket, game: &game::GameState) -> anyhow::Result<()> {
    let mut update = game.update.subscribe();
    let polling_interval = std::time::Duration::from_millis(100);

    loop {
        tokio::select! {
            msg = socket.recv() => {
                let msg = if let Some(msg) = msg {
                    msg
                } else {
                    // connection closed
                    return Ok(());
                };
                if let Message::Text(data) = msg? {
                    let msg: WsClientMessage = serde_json::from_str(&data)?;
                    handle_client_msg(msg, game).await?;
                }
            }
            _ = update.recv() => {
                send_game_state_update(socket, game).await?;
            }
            _ = tokio::time::sleep(polling_interval) => {
                let mut current_question = game.current_question.lock();
                if current_question.timer.elapsed() >= Duration::from_secs(10) {
                    handle_question_end(&mut current_question, game);
                }
            }
        }
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

    Router::new()
        .route("/game", post(new_game))
        .route("/game/:id", get(get_game_ws))
        .route("/search/:query", get(search_playlist))
        .with_state(state)
}
