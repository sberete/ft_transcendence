mod handlers;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

pub fn router() -> Router<PgPool>
{
    Router::new()
        .route("/", post(handlers::create_room))
        .route("/{room_id}", get(handlers::get_room).delete(handlers::delete_room))
        .route("/{room_id}/messages", get(handlers::list_messages))
}
