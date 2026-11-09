// Live in-memory room state. The database keeps the durable history
// (messages table) and the list of rooms (rooms table); this registry
// only knows what's useful while at least one connection is open: who's
// present, which video, at what position. It disappears and rebuilds
// from nothing on every server restart - that's intentional, nothing
// here is a durable source of truth.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use tokio::sync::mpsc;

use super::protocol::{now_ms, MediaRef, PlaybackRef, RoomRef, UserRef};

pub type OutgoingTx = mpsc::UnboundedSender<Message>;

struct LiveMember {
    user: UserRef,
    tx: OutgoingTx,
    // Used to find who has been present longest, to hand them room
    // ownership if the owner leaves (see leave()).
    joined_at: i64,
}

struct LiveRoom {
    name: String,
    owner_id: i64,
    members: HashMap<i64, LiveMember>,
    media: Option<MediaRef>,
    playing: bool,
    position: f64,
    server_time: i64,
}

pub struct JoinOutcome {
    pub room: RoomRef,
    pub members: Vec<UserRef>,
    pub media: Option<MediaRef>,
    pub playback: PlaybackRef,
    pub notify_others: Vec<OutgoingTx>,
}

pub struct LeaveOutcome {
    pub room_is_empty: bool,
    pub notify_remaining: Vec<OutgoingTx>,
    // Some(id) if whoever left was the owner and the room is still alive:
    // the new owner (already applied in the registry), to persist to the
    // database and announce to the remaining members.
    pub new_owner_id: Option<i64>,
}

#[derive(Clone)]
pub struct RoomRegistry {
    rooms: Arc<Mutex<HashMap<i64, LiveRoom>>>,
}

impl RoomRegistry {
    // Creates an empty registry; rooms are created lazily on first join.
    pub fn new() -> Self {
        RoomRegistry { rooms: Arc::new(Mutex::new(HashMap::new())) }
    }

    // Creates the in-memory entry if this is the first member to join
    // this room since the server started (name/owner_id are only used in
    // that case: they come from a DB read the caller just did).
    //
    // The returned `members` snapshot is taken BEFORE the new member is
    // inserted below, so it deliberately excludes the joiner themselves -
    // the frontend adds itself to the list client-side (see
    // frontend/src/services/roomSocket.js) rather than receiving a
    // self-referential list back from the server.
    pub fn join(&self, room_id: i64, code: &str, name: &str, owner_id: i64, user: UserRef, tx: OutgoingTx)
        -> JoinOutcome
    {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.entry(room_id).or_insert_with(|| LiveRoom {
            name: name.to_string(),
            owner_id,
            members: HashMap::new(),
            media: None,
            playing: false,
            position: 0.0,
            server_time: now_ms(),
        });

        let notify_others = room.members.values().map(|m| m.tx.clone()).collect();
        let members = room.members.values().map(|m| m.user.clone()).collect();
        let outcome = JoinOutcome {
            room: RoomRef { code: code.to_string(), name: room.name.clone(), owner_id: room.owner_id },
            members,
            media: room.media.clone(),
            playback: PlaybackRef { playing: room.playing, position: room.position, server_time: room.server_time },
            notify_others,
        };

        room.members.insert(user.id, LiveMember { user, tx, joined_at: now_ms() });
        outcome
    }

    // Removes a member, transferring ownership to the longest-present
    // remaining member if the owner was the one leaving, and deletes the
    // room entirely once it has no members left.
    pub fn leave(&self, room_id: i64, user_id: i64)
        -> LeaveOutcome
    {
        let mut rooms = self.rooms.lock().unwrap();
        let Some(room) = rooms.get_mut(&room_id) else {
            return LeaveOutcome { room_is_empty: true, notify_remaining: Vec::new(), new_owner_id: None };
        };

        let was_owner = room.owner_id == user_id;
        room.members.remove(&user_id);
        let room_is_empty = room.members.is_empty();

        // The owner leaves but the room goes on: it passes to whoever has
        // been present longest, rather than being stuck with no one able
        // to change the video.
        let new_owner_id = if was_owner && !room_is_empty {
            let heir = room.members.values().min_by_key(|m| m.joined_at).map(|m| m.user.id);
            if let Some(id) = heir {
                room.owner_id = id;
            }
            heir
        } else {
            None
        };

        let notify_remaining = room.members.values().map(|m| m.tx.clone()).collect();

        if room_is_empty {
            rooms.remove(&room_id);
        }

        LeaveOutcome { room_is_empty, notify_remaining, new_owner_id }
    }

    // Returns the outgoing channels for every current member of a room.
    pub fn members_tx(&self, room_id: i64)
        -> Vec<OutgoingTx>
    {
        self.rooms
            .lock()
            .unwrap()
            .get(&room_id)
            .map(|r| r.members.values().map(|m| m.tx.clone()).collect())
            .unwrap_or_default()
    }

    // Changes a room's current video, resetting playback to paused at position 0.
    pub fn set_media(&self, room_id: i64, media: MediaRef)
        -> Option<(PlaybackRef, MediaRef, Vec<OutgoingTx>)>
    {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.get_mut(&room_id)?;

        room.media = Some(media.clone());
        room.playing = false;
        room.position = 0.0;
        room.server_time = now_ms();

        let snapshot = PlaybackRef { playing: room.playing, position: room.position, server_time: room.server_time };
        let notify = room.members.values().map(|m| m.tx.clone()).collect();
        Some((snapshot, media, notify))
    }

    // play/pause/seek share the same shape: set a new reference point
    // (position, server_time) and broadcast it to everyone.
    pub fn set_playback(&self, room_id: i64, playing: bool, position: f64)
        -> Option<(PlaybackRef, Option<MediaRef>, Vec<OutgoingTx>)>
    {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.get_mut(&room_id)?;

        room.playing = playing;
        room.position = position;
        room.server_time = now_ms();

        let snapshot = PlaybackRef { playing: room.playing, position: room.position, server_time: room.server_time };
        let notify = room.members.values().map(|m| m.tx.clone()).collect();
        Some((snapshot, room.media.clone(), notify))
    }

    // Like set_playback, but leaves `playing` untouched: "jumps without
    // changing play/pause" (contract section 5).
    pub fn seek(&self, room_id: i64, position: f64)
        -> Option<(PlaybackRef, Option<MediaRef>, Vec<OutgoingTx>)>
    {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.get_mut(&room_id)?;

        room.position = position;
        room.server_time = now_ms();

        let snapshot = PlaybackRef { playing: room.playing, position: room.position, server_time: room.server_time };
        let notify = room.members.values().map(|m| m.tx.clone()).collect();
        Some((snapshot, room.media.clone(), notify))
    }
}
