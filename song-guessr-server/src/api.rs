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
    Ended,
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

            if room.users.get(&user_id).is_none() {
                room.users
                    .insert(user_id.clone(), game::User::new(user_name));
                let _ = room.update_broadcast.send(());
            }

            // TODO: properly handle the error
            let _result = handle_socket(&mut socket, &room, update_rx).await;

            room.users.remove(&user_id);
            let _ = room.update_broadcast.send(());
        })
    } else {
        (StatusCode::NOT_FOUND, "Room not found").into_response()
    }
}

async fn on_game_state_update(socket: &mut WebSocket, room: &game::Room) -> anyhow::Result<()> {
    let msg = {
        let game = room.game.read();
        match &*game {
            game::GameState::Waiting => {
                let users = room.users.iter().map(|u| u.value().clone()).collect();
                let msg = WsServerMessage::Waiting { users };
                let data = serde_json::to_string(&msg)?;
                Some(Message::Text(data))
            }
            _ => None,
        }
    };

    if let Some(msg) = msg {
        socket.send(msg).await?;
    }
    // let current_question_id = game.current_question.lock().id;
    // match game.questions.get(current_question_id) {
    //     Some(question) => {
    //         let users = game.users.iter().map(|u| u.value().clone()).collect();
    //         let msg = WsServerMessage::GameState {
    //             question: question.clone(),
    //             question_id: current_question_id,
    //             users,
    //         };
    //         let data = serde_json::to_string(&msg)?;
    //         socket.send(Message::Text(data)).await?;
    //     }
    //     None => {
    //         socket
    //             .send(Message::Text(serde_json::to_string(
    //                 &WsServerMessage::GameEnded,
    //             )?))
    //             .await?;
    //     }
    // }
    Ok(())
}

// fn handle_question_end(
//     current_question: &mut parking_lot::MutexGuard<game::QuestionState>,
//     game: &game::GameState,
// ) {
//     let submissions = mem::take(&mut current_question.submissions);
//     for submission in submissions {
//         if let Some(mut user) = game.users.get_mut(&submission.user_id) {
//             if submission.choice == game.questions[current_question.id].ans_id {
//                 user.score += 1;
//             }
//         }
//     }

//     current_question.id += 1;
//     current_question.timer = Instant::now();

//     let _ = game.update.send(()); // ignore broadcast send error
// }

async fn handle_client_msg(msg: WsClientMessage, room: &game::Room) -> anyhow::Result<()> {
    match msg {
        // WsClientMessage::UserSubmitted(submission) => {
        //     let mut current_question = game.current_question.lock();
        //     current_question.submissions.push(submission);

        //     // end the current question if all users have submitted
        //     if current_question.submissions.len() == game.users.len() {
        //         handle_question_end(&mut current_question, game);
        //     }
        // }
        _ => {}
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
            //     let mut current_question = game.current_question.lock();
            //     if current_question.timer.elapsed() >= Duration::from_secs(10) {
            //         handle_question_end(&mut current_question, game);
            //     }
            // }
        }
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
        .route("/search", get(search_playlist))
        .with_state(state)
}
