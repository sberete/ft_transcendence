mod handlers;

use axum::routing::post;
use axum::Router;
use sqlx::PgPool;

pub fn router()
    -> Router<PgPool>
{
    Router::new()
        .route("/me", axum::routing::get(handlers::get_me).patch(handlers::update_me))
        .route("/me/avatar", post(handlers::upload_avatar))
}
