// Real-time module: the single WebSocket endpoint behind rooms, chat
// and playback sync. `handler` implements the protocol, `registry`
// holds the in-memory live-room state, `protocol` defines the wire
// message types shared between them.

mod handler;
mod protocol;
mod registry;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

pub use registry::RoomRegistry;

// State specific to this sub-router: the pool (for the handler's DB
// queries) and the live-room registry, plus the allowed origins for
// manual validation (WebSockets aren't covered by CORS). Resolved with
// with_state() right here, then nest()ed into a parent Router<PgPool>
// which never needs to know about this state type.
#[derive(Clone)]
pub struct WsState {
    pub pool: PgPool,
    pub rooms: RoomRegistry,
    pub frontend_origins: Vec<String>,
}

// Builds the single `/` WebSocket route with its own resolved state.
pub fn router(pool: PgPool, rooms: RoomRegistry, frontend_origins: Vec<String>)
    -> Router<PgPool>
{
    let ws_router = Router::new()
        .route("/", get(handler::upgrade))
        .with_state(WsState { pool, rooms, frontend_origins });

    // A Router that's already resolved (state ()) can be nested inside a
    // Router<PgPool> without the latter having to share the same state type.
    Router::new().merge(ws_router)
}
