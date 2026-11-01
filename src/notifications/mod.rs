mod handlers;

use axum::routing::{get, post};
use axum::Router;
use sqlx::PgPool;

pub use handlers::create;

pub fn router() -> Router<PgPool> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/read-all", post(handlers::mark_all_read))
        .route("/{id}/read", post(handlers::mark_read))
}
