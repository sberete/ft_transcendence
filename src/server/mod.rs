use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

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

    let port = config.port;
    HttpServer::new(move || {
        // TODO Phase 5 : restreindre aux origines connues (plus de permissive() en prod)
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(pool.clone()))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
