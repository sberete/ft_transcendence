use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

const LIST_LIMIT: i64 = 50;

#[derive(Serialize, sqlx::FromRow)]
pub struct NotificationView {
    id: i64,
    kind: String,
    payload: Value,
    read_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

pub async fn list(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<Vec<NotificationView>>, ApiError>
{
    let notifications: Vec<NotificationView> = sqlx::query_as(
        "SELECT id, kind, payload, read_at, created_at
         FROM notifications
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT $2",
    )
    .bind(auth.user_id)
    .bind(LIST_LIMIT)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(notifications))
}

pub async fn mark_read(auth: AuthUser, State(pool): State<PgPool>, Path(id): Path<i64>)
    -> Result<StatusCode, ApiError>
{
    let updated = sqlx::query("UPDATE notifications SET read_at = now() WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    if updated.rows_affected() == 0 {
        return Err(ApiError::NotFound("notification not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn mark_all_read(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<StatusCode, ApiError>
{
    sqlx::query("UPDATE notifications SET read_at = now() WHERE user_id = $1 AND read_at IS NULL")
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

// Utilise par les autres modules (friends, rooms, ws) pour creer une
// notification. Best-effort partout ou c'est appele : un echec ici ne doit
// jamais faire echouer l'action principale (demande d'ami, creation de
// salon...), donc les appelants loggent et ignorent l'erreur plutot que de
// la propager.
pub async fn create(pool: &PgPool, user_id: i64, kind: &str, payload: Value) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO notifications (user_id, kind, payload) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(kind)
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(())
}
