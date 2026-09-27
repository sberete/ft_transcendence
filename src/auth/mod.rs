mod extractor;
mod handlers;
mod password;
mod session;

use axum::routing::post;
use axum::Router;
use sqlx::PgPool;

pub use extractor::AuthUser;

pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/signup", post(handlers::signup))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
}
