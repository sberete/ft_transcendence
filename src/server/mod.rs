use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::HeaderValue;
use axum::Extension;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::auth::{LoginRateLimiter, OAuthConfig};
use crate::config::Config;
use crate::db;
use crate::media::MediaClient;
use crate::routes;
use crate::ws::RoomRegistry;

pub async fn run() -> std::io::Result<()>
{
    let config = Config::init();

    let pool = db::init_pool(&config.database_url)
        .await
        .expect("failed to connect to the database");

    db::run_migrations(&pool)
        .await
        .expect("failed to run database migrations");

    log::info!("Starting server on 0.0.0.0:{}", config.port);

    let rooms = RoomRegistry::new();
    let frontend_origins = config.frontend_origins();

    // Mêmes origines que la vérification Origin du WebSocket (config::
    // frontend_origins) : seuls les fronts connus peuvent appeler l'API,
    // pas "*". Le joker ne couvre de toute façon pas Authorization.
    let cors_origins: Vec<HeaderValue> = frontend_origins
        .iter()
        .filter_map(|origin| HeaderValue::from_str(origin).ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods(Any)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let app = routes::router(pool.clone(), rooms, frontend_origins)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(Extension(MediaClient::new(&config.peertube_url)))
        .layer(Extension(OAuthConfig::new(&config)))
        .layer(Extension(LoginRateLimiter::new()))
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    axum::serve(listener, app).await
}
