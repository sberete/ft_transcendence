// Friends module: send/accept friend requests and remove friends. Also
// the source of the "a friend started a session" and friend-request
// notifications fired from `notifications::create`.

mod handlers;

use axum::routing::{delete, get, post};
use axum::Router;
use sqlx::PgPool;

// REST routes for listing friends/requests and acting on them.
pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/", get(handlers::list_friends).post(handlers::send_request))
        .route("/requests", get(handlers::list_requests))
        .route("/requests/{from_user_id}/accept", post(handlers::accept_request))
        .route("/{friend_id}", delete(handlers::remove_friend))
}
