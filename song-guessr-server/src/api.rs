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
struct NewRoomRequest {
    user_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewRoomResponse {
    room_id: String,
}

async fn new_room(
    State(state): State<Arc<AppState>>,
    Json(request): Json<NewRoomRequest>,
) -> Result<Json<NewRoomResponse>, AppError> {
    let NewRoomRequest { user_id } = request;
    let room = game::Room::new(user_id);
    let room_id = game::gen_id(8);
    state.rooms.insert(room_id.clone(), Arc::new(room));
    Ok(Json(NewRoomResponse { room_id }))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    NewGame,
    UserSubmitted(game::UserSubmission),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    Waiting {
        users: Vec<game::User>,
    },
    Playing {
        question: game::Question,
        question_id: usize,
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
                let users = room.active_users();
                let msg = WsServerMessage::Waiting { users };
                let data = serde_json::to_string(&msg)?;
                Some(Message::Text(data))
            }
            game::GameState::Playing(state) => {
                let users = room.active_users();
                let msg = WsServerMessage::Playing {
                    question: state.questions[state.current_question.id].clone(),
                    question_id: state.current_question.id,
                    users,
                };
                let data = serde_json::to_string(&msg)?;
                Some(Message::Text(data))
            }
            &game::GameState::Ended => {
                let users = room.active_users();
                let msg = WsServerMessage::Ended { users };
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
        WsClientMessage::NewGame => {
            let mut game = room.game.write();
            *game = game::GameState::Waiting;
            let _ = room.update_broadcast.send(());
        }
        WsClientMessage::UserSubmitted(submission) => {
            let mut game = room.game.write();
            if let game::GameState::Playing(state) = &mut (*game) {
                state.current_question.submissions.push(submission);
                // end the current question if all users have submitted
                if state.current_question.submissions.len() == room.users.len() {
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
    // let polling_interval = std::time::Duration::from_millis(100);

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
                // _ = tokio::time::sleep(polling_interval) => {
                // room.periodic_update();
            // }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct NewGameRequest {
    playlist_id: String,
    num_questions: Option<usize>,
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
    } = request;

    let num_questions = num_questions.unwrap_or(15);
    let tracks = state.client.playlist_tracks(playlist_id).await?;
    let questions = game::gen_questions(tracks, num_questions);

    room.new_game(questions);

    Ok(Json(()))
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
        .route("/room/:id/game", post(new_game))
        .route("/search", get(search_playlist))
        .with_state(state)
}
