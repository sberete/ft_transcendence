mod db;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use sqlx::postgres::PgPool;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
    database: bool,
}

async fn health(pool: web::Data<PgPool>) -> impl Responder {
    let database_ok = sqlx::query("SELECT 1").execute(pool.get_ref()).await.is_ok();

    HttpResponse::Ok().json(HealthStatus {
        status: "ok",
        database: database_ok,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");

    let pool = db::init_pool(&database_url)
        .await
        .expect("failed to connect to the database");

    log::info!("Starting server on 0.0.0.0:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/health", web::get().to(health))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
