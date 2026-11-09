// User profile module: reading/updating one's own profile, and
// uploading or deleting an avatar image.

mod handlers;

use axum::routing::post;
use axum::Router;
use sqlx::PgPool;

// REST routes for the current user's own profile and avatar.
pub fn router() -> Router<PgPool>
{
    Router::new()
        .route("/me", axum::routing::get(handlers::get_me).patch(handlers::update_me))
        .route("/me/avatar", post(handlers::upload_avatar).delete(handlers::delete_avatar))
}
