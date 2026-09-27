use axum::Router;
use sqlx::PgPool;

// Chaque phase ajoute son propre sous-module ici (auth, users, friends, ws, media...)
// et le nest() ici.
pub fn router() -> Router<PgPool> {
    Router::new()
        .nest("/auth", crate::auth::router())
        .nest("/users", crate::users::router())
}
