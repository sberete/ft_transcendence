// Persistent notifications module: friend requests/accepts/removals, a
// friend starting a session, and room ownership transfers. Fetched by
// the frontend via polling - `create` is the helper other modules call
// to fire one off.

mod handlers;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

pub use handlers::create;

// REST routes: list, mark one read, mark all read.
pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/read-all", post(handlers::mark_all_read))
        .route("/{id}/read", post(handlers::mark_read))
}
