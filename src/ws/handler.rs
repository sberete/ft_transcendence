use std::time::{Duration, Instant};

use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use sqlx::PgPool;
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::protocol::{now_ms, ClientMessage, MediaRef, ServerMessage, UserRef};
use super::registry::OutgoingTx;
use super::WsState;
use crate::auth::validate_session;
use crate::media::is_valid_media_ref;

const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_FRAME_BYTES: usize = 4 * 1024;
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(60);
const CHAT_LIMIT: (u32, Duration) = (5, Duration::from_secs(5));
const PLAYBACK_LIMIT: (u32, Duration) = (10, Duration::from_secs(5));
const MAX_RATE_VIOLATIONS: u32 = 5;

pub async fn upgrade(ws: WebSocketUpgrade, State(state): State<WsState>, headers: HeaderMap)
    -> Response
{
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    let allowed = origin.is_some_and(|o| state.frontend_origins.iter().any(|allowed| allowed == o));

    if !allowed {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    // Tache dediee a l'ecriture : tout le reste pousse dans `tx` au lieu
    // d'ecrire directement sur le socket, pour pouvoir recevoir des messages
    // venant d'AUTRES connexions (diffusion) sans se les disputer.
    let writer = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });

    let close_code = run_connection(&mut stream, &tx, &state).await;
    let _ = tx.send(Message::Close(Some(CloseFrame {
        code: close_code,
        reason: close_reason(close_code).into(),
    })));
    drop(tx);
    let _ = writer.await;
}

fn close_reason(code: u16) -> &'static str {
    match code {
        4400 => "bad message",
        4401 => "unauthorized",
        4408 => "inactivity",
        4429 => "rate limited",
        _ => "normal closure",
    }
}

async fn run_connection(stream: &mut SplitStream<WebSocket>, tx: &OutgoingTx, state: &WsState) -> u16 {
    let user = match authenticate_first_message(stream, &state.pool).await {
        Some(user) => user,
        None => return 4401,
    };
    send(tx, &ServerMessage::AuthOk { user: user.clone() });

    let mut current_room: Option<(i64, i64)> = None; // (room_id, owner_id)
    let mut chat_limit = RateLimiter::new(CHAT_LIMIT.0, CHAT_LIMIT.1);
    let mut playback_limit = RateLimiter::new(PLAYBACK_LIMIT.0, PLAYBACK_LIMIT.1);
    let mut violations = 0u32;

    let close_code = loop {
        let frame = match timeout(INACTIVITY_TIMEOUT, stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => text,
            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => break 1000,
            Ok(Some(Ok(_))) => continue, // ping/pong/binary : ignores
            Ok(Some(Err(_))) => break 1000,
            Err(_) => break 4408, // inactivite
        };

        if frame.len() > MAX_FRAME_BYTES {
            break 4400;
        }

        let message: ClientMessage = match serde_json::from_str(&frame) {
            Ok(m) => m,
            Err(_) => {
                send(tx, &ServerMessage::error("bad_message", "invalid JSON or unknown type"));
                continue;
            }
        };

        let within_limit = match message {
            ClientMessage::ChatSend { .. } => chat_limit.allow(),
            ClientMessage::PlaybackSetMedia { .. }
            | ClientMessage::PlaybackPlay { .. }
            | ClientMessage::PlaybackPause { .. }
            | ClientMessage::PlaybackSeek { .. } => playback_limit.allow(),
            _ => true,
        };

        if !within_limit {
            violations += 1;
            send(tx, &ServerMessage::error("rate_limited", "slow down"));
            if violations >= MAX_RATE_VIOLATIONS {
                break 4429;
            }
            continue;
        }

        match message {
            ClientMessage::Auth { .. } => {
                send(tx, &ServerMessage::error("bad_message", "already authenticated"));
            }
            ClientMessage::RoomJoin { code } => {
                handle_join(state, tx, &user, &mut current_room, code).await;
            }
            ClientMessage::RoomLeave => {
                leave_current_room(state, tx, &user, &mut current_room).await;
            }
            ClientMessage::ChatSend { content } => {
                handle_chat_send(state, tx, &user, current_room, content).await;
            }
            ClientMessage::PlaybackSetMedia { provider, id } => {
                handle_set_media(state, tx, user.id, current_room, provider, id).await;
            }
            ClientMessage::PlaybackPlay { position } => {
                handle_playback(state, tx, current_room, user.id, true, position).await;
            }
            ClientMessage::PlaybackPause { position } => {
                handle_playback(state, tx, current_room, user.id, false, position).await;
            }
            ClientMessage::PlaybackSeek { position } => {
                handle_seek(state, tx, current_room, user.id, position).await;
            }
            ClientMessage::Ping { t } => {
                send(tx, &ServerMessage::Pong { t, server_time: now_ms() });
            }
        }
    };

    leave_current_room(state, tx, &user, &mut current_room).await;
    close_code
}

// Le premier message doit etre `auth` dans les 5 secondes, sinon 4401.
async fn authenticate_first_message(stream: &mut SplitStream<WebSocket>, pool: &PgPool) -> Option<UserRef> {
    let Ok(Some(Ok(Message::Text(text)))) = timeout(AUTH_TIMEOUT, stream.next()).await else {
        return None;
    };
    if text.len() > MAX_FRAME_BYTES {
        return None;
    }
    let ClientMessage::Auth { token } = serde_json::from_str(&text).ok()? else {
        return None;
    };

    let user_id = validate_session(pool, &token).await.ok().flatten()?;
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT username, avatar_url FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .ok()?;
    let (username, avatar_url) = row?;

    Some(UserRef { id: user_id, username, avatar_url })
}

async fn handle_join(
    state: &WsState,
    tx: &OutgoingTx,
    user: &UserRef,
    current_room: &mut Option<(i64, i64)>,
    code: String,
)
{
    leave_current_room(state, tx, user, current_room).await;

    // code -> id interne : le registre en memoire est indexe par l'id (jamais
    // expose), seul le code voyage avec le client.
    let room_row: Option<(i64, String, i64)> =
        sqlx::query_as("SELECT id, name, owner_id FROM rooms WHERE code = $1")
            .bind(&code)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

    let Some((room_id, name, owner_id)) = room_row else {
        send(tx, &ServerMessage::error("room_not_found", "this room does not exist"));
        return;
    };

    let outcome = state.rooms.join(room_id, &code, &name, owner_id, user.clone(), tx.clone());
    *current_room = Some((room_id, owner_id));

    broadcast(&outcome.notify_others, &ServerMessage::RoomMemberJoined { user: user.clone() });
    send(
        tx,
        &ServerMessage::RoomState {
            room: outcome.room,
            members: outcome.members,
            media: outcome.media,
            playback: outcome.playback,
        },
    );
}

async fn leave_current_room(state: &WsState, _tx: &OutgoingTx, user: &UserRef, current_room: &mut Option<(i64, i64)>) {
    let Some((room_id, _owner_id)) = current_room.take() else { return };

    let outcome = state.rooms.leave(room_id, user.id);
    if outcome.room_is_empty {
        let _ = sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(room_id)
            .execute(&state.pool)
            .await;
    } else {
        if let Some(new_owner_id) = outcome.new_owner_id {
            let _ = sqlx::query("UPDATE rooms SET owner_id = $1 WHERE id = $2")
                .bind(new_owner_id)
                .bind(room_id)
                .execute(&state.pool)
                .await;
        }
        broadcast(
            &outcome.notify_remaining,
            &ServerMessage::RoomMemberLeft { user_id: user.id, new_owner_id: outcome.new_owner_id },
        );
    }
}

async fn handle_chat_send(state: &WsState, tx: &OutgoingTx, user: &UserRef, current_room: Option<(i64, i64)>, content: String) {
    let Some((room_id, _)) = current_room else {
        send(tx, &ServerMessage::error("not_in_room", "join a room first"));
        return;
    };

    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 500 {
        send(tx, &ServerMessage::error("bad_message", "content must be 1 to 500 characters"));
        return;
    }

    let row: Result<(i64, chrono::DateTime<chrono::Utc>), _> = sqlx::query_as(
        "INSERT INTO messages (room_id, sender_id, content) VALUES ($1, $2, $3) RETURNING id, created_at",
    )
    .bind(room_id)
    .bind(user.id)
    .bind(trimmed)
    .fetch_one(&state.pool)
    .await;

    let Ok((id, created_at)) = row else {
        send(tx, &ServerMessage::error("bad_message", "failed to send message"));
        return;
    };

    let message = ServerMessage::ChatMessage {
        id,
        sender: user.clone(),
        content: trimmed.to_string(),
        created_at,
    };
    broadcast(&state.rooms.members_tx(room_id), &message);
}

async fn handle_set_media(
    state: &WsState,
    tx: &OutgoingTx,
    user_id: i64,
    current_room: Option<(i64, i64)>,
    provider: crate::media::ProviderId,
    id: String,
)
{
    let Some((room_id, owner_id)) = current_room else {
        send(tx, &ServerMessage::error("not_in_room", "join a room first"));
        return;
    };
    if user_id != owner_id {
        send(tx, &ServerMessage::error("forbidden", "only the room owner can change the video"));
        return;
    }
    if !is_valid_media_ref(provider, &id) {
        send(tx, &ServerMessage::error("invalid_media", "invalid provider/id"));
        return;
    }

    if let Some((snapshot, media, notify)) = state.rooms.set_media(room_id, MediaRef { provider, id }) {
        broadcast_playback_state(&notify, snapshot, Some(media), user_id);
    }
}

async fn handle_playback(
    state: &WsState,
    tx: &OutgoingTx,
    current_room: Option<(i64, i64)>,
    user_id: i64,
    playing: bool,
    position: f64,
)
{
    let Some((room_id, _)) = current_room else {
        send(tx, &ServerMessage::error("not_in_room", "join a room first"));
        return;
    };

    if let Some((snapshot, media, notify)) = state.rooms.set_playback(room_id, playing, position) {
        broadcast_playback_state(&notify, snapshot, media, user_id);
    }
}

async fn handle_seek(state: &WsState, tx: &OutgoingTx, current_room: Option<(i64, i64)>, user_id: i64, position: f64) {
    let Some((room_id, _)) = current_room else {
        send(tx, &ServerMessage::error("not_in_room", "join a room first"));
        return;
    };

    if let Some((snapshot, media, notify)) = state.rooms.seek(room_id, position) {
        broadcast_playback_state(&notify, snapshot, media, user_id);
    }
}

fn broadcast_playback_state(notify: &[OutgoingTx], snapshot: super::protocol::PlaybackRef, media: Option<MediaRef>, by: i64) {
    broadcast(
        notify,
        &ServerMessage::PlaybackState {
            media,
            playing: snapshot.playing,
            position: snapshot.position,
            server_time: snapshot.server_time,
            by,
        },
    );
}

fn send(tx: &OutgoingTx, message: &ServerMessage) {
    if let Ok(text) = serde_json::to_string(message) {
        let _ = tx.send(Message::Text(text.into()));
    }
}

fn broadcast(targets: &[OutgoingTx], message: &ServerMessage) {
    let Ok(text) = serde_json::to_string(message) else { return };
    for target in targets {
        let _ = target.send(Message::Text(text.clone().into()));
    }
}

struct RateLimiter {
    max: u32,
    window: Duration,
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    fn new(max: u32, window: Duration) -> Self {
        RateLimiter { max, window, count: 0, window_start: Instant::now() }
    }

    fn allow(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.window_start) >= self.window {
            self.window_start = now;
            self.count = 0;
        }
        if self.count >= self.max {
            false
        } else {
            self.count += 1;
            true
        }
    }
}
