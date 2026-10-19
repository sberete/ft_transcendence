mod handler;
mod protocol;
mod registry;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

pub use registry::RoomRegistry;

// Etat propre a ce sous-routeur : le pool (pour les requetes DB du handler)
// et le registre des salons vivants, plus les origines autorisees pour la
// verification manuelle (les WebSockets ne passent pas par le CORS). Resolu
// avec with_state() ici meme, puis nest() dans un Router<PgPool> parent sans
// que celui-ci ait besoin de connaitre ce type d'etat.
#[derive(Clone)]
pub struct WsState {
    pub pool: PgPool,
    pub rooms: RoomRegistry,
    pub frontend_origins: Vec<String>,
}

pub fn router(pool: PgPool, rooms: RoomRegistry, frontend_origins: Vec<String>)
    -> Router<PgPool>
{
    let ws_router = Router::new()
        .route("/", get(handler::upgrade))
        .with_state(WsState { pool, rooms, frontend_origins });

    // Un Router deja resolu (etat ()) peut etre imbrique dans un Router<PgPool>
    // sans que ce dernier ait a partager le meme type d'etat.
    Router::new().merge(ws_router)
}
