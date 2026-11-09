// PostgreSQL connection setup: a pool shared across the whole application
// (injected as Axum State) and running SQL migrations at startup. No
// business queries here, just the SQLx plumbing.

use sqlx::postgres::{PgPool, PgPoolOptions};

// Creates the shared connection pool used by every module.
pub async fn init_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}

// Applies any SQL migrations (migrations/ folder) that haven't run yet.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
