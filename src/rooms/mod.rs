// Rooms module: the one-off REST actions around a room (creating it,
// reading its metadata, reading chat history). The live, real-time
// side (joining, playback sync, presence) lives in `ws`, not here.

mod handlers;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

// REST routes for rooms: create, read, and read chat history.
pub fn router() -> Router<PgPool>
{
    Router::new()
        .route("/", post(handlers::create_room))
        .route("/{room_id}", get(handlers::get_room))
        .route("/{room_id}/messages", get(handlers::list_messages))
}
