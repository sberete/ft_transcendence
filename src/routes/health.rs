use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
    database: bool,
}

pub async fn health(pool: web::Data<PgPool>) -> impl Responder {
    let database_ok = sqlx::query("SELECT 1").execute(pool.get_ref()).await.is_ok();

    HttpResponse::Ok().json(HealthStatus {
        status: "ok",
        database: database_ok,
    })
}
