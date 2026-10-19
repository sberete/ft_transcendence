// Etat vivant des salons, en memoire. La DB garde l'historique (table
// messages) et la liste des salons (table rooms) ; ce registre, lui, ne
// connait que ce qui est utile tant qu'au moins une connexion est ouverte :
// qui est present, quelle video, a quelle position. Il disparait et se
// reconstruit a partir de rien a chaque redemarrage du serveur - c'est
// voulu, rien ici n'est source de verite durable.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::Message;
use tokio::sync::mpsc;

use super::protocol::{now_ms, MediaRef, PlaybackRef, RoomRef, UserRef};

pub type OutgoingTx = mpsc::UnboundedSender<Message>;

struct LiveMember {
    user: UserRef,
    tx: OutgoingTx,
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
}

#[derive(Clone)]
pub struct RoomRegistry {
    rooms: Arc<Mutex<HashMap<i64, LiveRoom>>>,
}

impl RoomRegistry {
    pub fn new() -> Self {
        RoomRegistry { rooms: Arc::new(Mutex::new(HashMap::new())) }
    }

    // Cree l'entree en memoire si c'est le premier membre a rejoindre cette
    // room depuis le demarrage du serveur (name/owner_id ne servent que dans
    // ce cas : ils viennent d'une lecture DB faite juste avant par l'appelant).
    pub fn join(&self, room_id: i64, name: &str, owner_id: i64, user: UserRef, tx: OutgoingTx)
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
            room: RoomRef { id: room_id, name: room.name.clone(), owner_id: room.owner_id },
            members,
            media: room.media.clone(),
            playback: PlaybackRef { playing: room.playing, position: room.position, server_time: room.server_time },
            notify_others,
        };

        room.members.insert(user.id, LiveMember { user, tx });
        outcome
    }

    pub fn leave(&self, room_id: i64, user_id: i64)
        -> LeaveOutcome
    {
        let mut rooms = self.rooms.lock().unwrap();
        let Some(room) = rooms.get_mut(&room_id) else {
            return LeaveOutcome { room_is_empty: true, notify_remaining: Vec::new() };
        };

        room.members.remove(&user_id);
        let room_is_empty = room.members.is_empty();
        let notify_remaining = room.members.values().map(|m| m.tx.clone()).collect();

        if room_is_empty {
            rooms.remove(&room_id);
        }

        LeaveOutcome { room_is_empty, notify_remaining }
    }

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

    // play/pause/seek partagent la meme forme : on pose un nouveau point de
    // reference (position, server_time) et on diffuse a tout le monde.
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

    // Comme set_playback, mais ne touche pas a `playing` : "saute sans changer
    // lecture/pause" (section 5 du contrat).
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
