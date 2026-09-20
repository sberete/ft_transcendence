use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;
use crate::db;
use crate::routes;

pub async fn run()
    -> std::io::Result<()>
{
    let config = Config::init();

    let pool = db::init_pool(&config.database_url)
        .await
        .expect("failed to connect to the database");

    db::run_migrations(&pool)
        .await
        .expect("failed to run database migrations");

    log::info!("Starting server on 0.0.0.0:{}", config.port);

    // Le joker "*" ne couvre pas Authorization : il faut le lister explicitement.
    // TODO Phase 5 : restreindre allow_origin aux origines connues
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

    let app = routes::router()
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port)).await?;
    axum::serve(listener, app).await
}
