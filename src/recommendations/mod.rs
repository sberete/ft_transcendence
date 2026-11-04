mod handlers;
mod scoring;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

pub fn router() -> Router<PgPool> {
    Router::new().route("/", get(handlers::list))
}
