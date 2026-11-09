// Recommendation module: hybrid collaborative + content-based scoring
// (see `handlers` for the pipeline and `scoring` for the hand-rolled
// TF-IDF/cosine-similarity model it uses for the content side).

mod handlers;
mod scoring;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

// Single route: the personalized recommendation list for the caller.
pub fn router() -> Router<PgPool> {
    Router::new().route("/", get(handlers::list))
}
