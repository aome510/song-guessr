use dashmap::DashMap;
use rspotify::model::SimplifiedPlaylist;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use crate::{client, game};

struct AppState {
    client: client::Client,
    rooms: DashMap<String, Arc<game::Room>>,
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
struct NewRoomResponse {
    room_id: String,
}

async fn new_room(State(state): State<Arc<AppState>>) -> Result<Json<NewRoomResponse>, AppError> {
    let room = game::Room::new();
    let room_id = game::gen_id(8);
    state.rooms.insert(room_id.clone(), Arc::new(room));
    Ok(Json(NewRoomResponse { room_id }))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    UserSubmitted(game::UserSubmission),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    WaitingForGame {
        users: Vec<game::User>,
    },
    Playing {
        question: game::Question,
        question_id: usize,
        song_progress_ms: u32,
        users: Vec<game::User>,
    },
    WaitingForNextQuestion {
        answer: String,
        correct_submissions: Vec<game::UserSubmission>,
        users: Vec<game::User>,
    },
    Ended {
        users: Vec<game::User>,
    },
}

#[derive(Debug, Deserialize)]
struct RoomParams {
    user_id: String,
    user_name: String,
}

async fn get_room_ws(
    Path(id): Path<String>,
    Query(params): Query<RoomParams>,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    let RoomParams { user_id, user_name } = params;

    if let Some(room) = state.rooms.get(&id) {
        let room = room.clone();

        ws.on_upgrade(move |mut socket| async move {
            let update_rx = room.update_broadcast.subscribe();

            room.on_user_join(&user_id, &user_name);

            // TODO: properly handle the error
            let _result = handle_socket(&mut socket, &room, update_rx).await;

            room.on_user_leave(
                &user_id,
                matches!(&*room.game.read(), &game::GameState::Waiting { .. }),
            );
        })
    } else {
        (StatusCode::NOT_FOUND, format!("Room {id} not found")).into_response()
    }
}

async fn on_game_state_update(socket: &mut WebSocket, room: &game::Room) -> anyhow::Result<()> {
    let msg = {
        let game = room.game.read();
        match &*game {
            game::GameState::Waiting => {
                let msg = WsServerMessage::WaitingForGame {
                    users: room.users(),
                };
                let data = serde_json::to_string(&msg)?;
                Some(Message::Text(data))
            }
            game::GameState::Playing(state) => match state.question_state.status {
                game::QuestionStatus::Playing => {
                    let msg = WsServerMessage::Playing {
                        question: state.current_question().clone(),
                        question_id: state.question_state.id,
                        song_progress_ms: state.question_state.timer.elapsed().as_millis() as u32,
                        users: room.users(),
                    };
                    let data = serde_json::to_string(&msg)?;
                    Some(Message::Text(data))
                }
                game::QuestionStatus::Ended => {
                    let current_question = state.current_question();
                    let msg = WsServerMessage::WaitingForNextQuestion {
                        answer: current_question.choices[current_question.ans_id].clone(),
                        correct_submissions: state
                            .question_state
                            .submissions
                            .iter()
                            .filter(|s| s.choice == current_question.ans_id)
                            .cloned()
                            .collect(),
                        users: room.users(),
                    };
                    let data = serde_json::to_string(&msg)?;
                    Some(Message::Text(data))
                }
            },
            &game::GameState::Ended { .. } => {
                let msg = WsServerMessage::Ended {
                    users: room.users(),
                };
                let data = serde_json::to_string(&msg)?;
                Some(Message::Text(data))
            }
        }
    };

    if let Some(msg) = msg {
        socket.send(msg).await?;
    }

    Ok(())
}

async fn handle_client_msg(msg: WsClientMessage, room: &game::Room) -> anyhow::Result<()> {
    match msg {
        WsClientMessage::UserSubmitted(submission) => {
            let mut game = room.game.write();
            if let game::GameState::Playing(state) = &mut (*game) {
                state.question_state.submissions.push(submission);
                // end the current question if all users have submitted
                if state.question_state.submissions.len() == room.users.len() {
                    drop(game);
                    room.on_question_end();
                }
            }
        }
    }
    Ok(())
}

async fn handle_socket(
    socket: &mut WebSocket,
    room: &game::Room,
    mut update_rx: tokio::sync::broadcast::Receiver<()>,
) -> anyhow::Result<()> {
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
                    handle_client_msg(msg, room).await?;
                }
            }
            _ = update_rx.recv() => {
                on_game_state_update(socket, room).await?;
            }
            _ = tokio::time::sleep(polling_interval) => {
                room.periodic_update();
            }
        }
    }
}

async fn reset_room(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, AppError> {
    if let Some(room) = state.rooms.get(&id) {
        room.users.retain(|_, u| u.online);
        for mut user in room.users.iter_mut() {
            user.score = 0;
        }
        let mut game = room.game.write();
        *game = game::GameState::Waiting;
        let _ = room.update_broadcast.send(());
        Ok(Json(()))
    } else {
        Err(anyhow::anyhow!("Room {id} not found").into())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewGameRequest {
    playlist_id: String,
    num_questions: Option<usize>,
    question_types: Vec<game::QuestionType>,
}

async fn new_game(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(request): Json<NewGameRequest>,
) -> Result<Json<()>, AppError> {
    let room = if let Some(room) = state.rooms.get(&id) {
        if !matches!(&*room.game.read(), game::GameState::Waiting) {
            return Err(anyhow::anyhow!("Game already in progress").into());
        }

        room.clone()
    } else {
        return Err(anyhow::anyhow!("Room {id} not found").into());
    };

    let NewGameRequest {
        playlist_id,
        num_questions,
        question_types,
    } = request;

    let num_questions = num_questions.unwrap_or(15);
    let tracks = state.client.playlist_tracks(&playlist_id).await?;
    let questions = game::gen_questions(tracks, num_questions, question_types.clone());

    room.new_game(playlist_id, question_types, questions);

    Ok(Json(()))
}

async fn restart_game(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<()>, AppError> {
    if let Some(room) = state.rooms.get(&id) {
        let (playlist_id, num_questions, question_types) = if let game::GameState::Ended {
            playlist_id,
            num_questions,
            question_types,
        } = &*room.game.read()
        {
            (playlist_id.clone(), *num_questions, question_types.clone())
        } else {
            return Err(anyhow::anyhow!("Game has not ended yet").into());
        };

        let tracks = state.client.playlist_tracks(&playlist_id).await?;
        let questions = game::gen_questions(tracks, num_questions, question_types.clone());
        room.new_game(playlist_id, question_types, questions);

        Ok(Json(()))
    } else {
        Err(anyhow::anyhow!("Room {id} not found").into())
    }
}
#[derive(Debug, Deserialize)]
struct SearchParams {
    query: String,
}

async fn search_playlist(
    Query(params): Query<SearchParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SimplifiedPlaylist>>, AppError> {
    Ok(state.client.search_playlist(params.query).await.map(Json)?)
}

pub fn new_app(client: client::Client) -> Router {
    let state = Arc::new(AppState {
        client,
        rooms: DashMap::new(),
    });

    Router::new()
        .route("/room", post(new_room))
        .route("/room/:id", get(get_room_ws))
        .route("/room/:id/reset", post(reset_room))
        .route("/room/:id/game", post(new_game))
        .route("/room/:id/restart", post(restart_game))
        .route("/search", get(search_playlist))
        .with_state(state)
}
