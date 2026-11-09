// HTTP handlers for listing and acknowledging notifications, plus the
// `create` helper used by other modules to fire one off.

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

// Lists this user's most recent notifications, newest first.
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

// Marks a single notification as read, scoped to its owner.
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

// Marks every unread notification for this user as read in one go.
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

// Used by other modules (friends, rooms, ws) to create a notification.
// Best-effort everywhere it's called: a failure here must never fail the
// main action (friend request, room creation...), so callers log and
// ignore the error instead of propagating it.
pub async fn create(pool: &PgPool, user_id: i64, kind: &str, payload: Value) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO notifications (user_id, kind, payload) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(kind)
        .bind(payload)
        .execute(pool)
        .await?;
    Ok(())
}
