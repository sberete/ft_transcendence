// Top-level route table: mounts every module's own router under its
// path prefix. Each new module gets one more `nest()` call here.

use axum::Router;
use sqlx::PgPool;

use crate::ws::RoomRegistry;

// Assembles the full REST + WebSocket router served by the backend.
pub fn router(pool: PgPool, rooms: RoomRegistry, frontend_origins: Vec<String>) -> Router<PgPool> {
    Router::new()
        .nest("/auth", crate::auth::router())
        .nest("/users", crate::users::router())
        .nest("/friends", crate::friends::router())
        .nest("/media", crate::media::router())
        .nest("/notifications", crate::notifications::router())
        .nest("/recommendations", crate::recommendations::router())
        .nest("/rooms", crate::rooms::router())
        .nest("/ws", crate::ws::router(pool, rooms, frontend_origins))
}
