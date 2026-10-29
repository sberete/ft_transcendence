// Formes des messages echanges sur /ws, miroir exact de docs/websocket.md.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::media::ProviderId;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },
    #[serde(rename = "room.join")]
    RoomJoin { code: String },
    #[serde(rename = "room.leave")]
    RoomLeave,
    #[serde(rename = "chat.send")]
    ChatSend { content: String },
    #[serde(rename = "playback.set_media")]
    PlaybackSetMedia { provider: ProviderId, id: String },
    #[serde(rename = "playback.play")]
    PlaybackPlay { position: f64 },
    #[serde(rename = "playback.pause")]
    PlaybackPause { position: f64 },
    #[serde(rename = "playback.seek")]
    PlaybackSeek { position: f64 },
    #[serde(rename = "ping")]
    Ping { t: i64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct UserRef {
    pub id: i64,
    pub username: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomRef {
    pub code: String,
    pub name: String,
    pub owner_id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaRef {
    pub provider: ProviderId,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackRef {
    pub playing: bool,
    pub position: f64,
    pub server_time: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "auth.ok")]
    AuthOk { user: UserRef },
    #[serde(rename = "room.state")]
    RoomState {
        room: RoomRef,
        members: Vec<UserRef>,
        media: Option<MediaRef>,
        playback: PlaybackRef,
    },
    #[serde(rename = "room.member_joined")]
    RoomMemberJoined { user: UserRef },
    #[serde(rename = "room.member_left")]
    RoomMemberLeft { user_id: i64 },
    // Pas encore declenche par ce code : la seule facon de fermer une room
    // pour l'instant est l'auto-suppression quand elle se vide (personne a
    // prevenir, donc inutile d'emettre ce message dans ce cas precis).
    // Garde pour completude du contrat - utile le jour ou DELETE /rooms/{id}
    // (branche rooms-rest) sera reliee a ce registre pour fermer une salle
    // encore occupee.
    #[serde(rename = "room.closed")]
    #[allow(dead_code)]
    RoomClosed,
    #[serde(rename = "chat.message")]
    ChatMessage {
        id: i64,
        sender: UserRef,
        content: String,
        created_at: DateTime<Utc>,
    },
    #[serde(rename = "playback.state")]
    PlaybackState {
        media: Option<MediaRef>,
        playing: bool,
        position: f64,
        server_time: i64,
        by: i64,
    },
    #[serde(rename = "pong")]
    Pong { t: i64, server_time: i64 },
    #[serde(rename = "error")]
    Error { code: &'static str, message: String },
}

impl ServerMessage {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        ServerMessage::Error { code, message: message.into() }
    }
}

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}
