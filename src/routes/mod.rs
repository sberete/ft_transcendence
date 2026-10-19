use axum::Router;
use sqlx::PgPool;

use crate::ws::RoomRegistry;

// Chaque phase ajoute son propre sous-module ici (auth, users, friends, ws, media...)
// et le nest() ici.
pub fn router(pool: PgPool, rooms: RoomRegistry, frontend_origins: Vec<String>) -> Router<PgPool> {
    Router::new()
        .nest("/auth", crate::auth::router())
        .nest("/users", crate::users::router())
        .nest("/friends", crate::friends::router())
        .nest("/media", crate::media::router())
        .nest("/ws", crate::ws::router(pool, rooms, frontend_origins))
}
