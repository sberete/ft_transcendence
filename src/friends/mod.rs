mod handlers;

use axum::routing::{delete, get};
use axum::Router;
use sqlx::PgPool;

pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/", get(handlers::list_friends).post(handlers::add_friend))
        .route("/{friend_id}", delete(handlers::remove_friend))
}
