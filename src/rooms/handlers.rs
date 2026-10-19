use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RoomView {
    id: i64,
    name: String,
    owner_id: i64,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct ListMessagesParams {
    before: Option<i64>,
    limit: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: i64,
    content: String,
    created_at: DateTime<Utc>,
    sender_id: i64,
    sender_username: String,
    sender_avatar_url: Option<String>,
}

#[derive(Serialize)]
struct SenderView {
    id: i64,
    username: String,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
pub struct ChatMessageView {
    id: i64,
    sender: SenderView,
    content: String,
    created_at: DateTime<Utc>,
}

impl From<MessageRow> for ChatMessageView {
    fn from(row: MessageRow) -> Self {
        ChatMessageView {
            id: row.id,
            sender: SenderView {
                id: row.sender_id,
                username: row.sender_username,
                avatar_url: row.sender_avatar_url,
            },
            content: row.content,
            created_at: row.created_at,
        }
    }
}

pub async fn create_room(auth: AuthUser, State(pool): State<PgPool>, Json(payload): Json<CreateRoomRequest>)
    -> Result<(StatusCode, Json<RoomView>), ApiError>
{
    if payload.name.trim().is_empty() {
        return Err(ApiError::BadRequest("room name is required".into()));
    }

    let room: RoomView = sqlx::query_as(
        "INSERT INTO rooms (name, owner_id) VALUES ($1, $2) RETURNING id, name, owner_id, created_at",
    )
    .bind(payload.name.trim())
    .bind(auth.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(room)))
}

pub async fn get_room(_auth: AuthUser, State(pool): State<PgPool>, Path(room_id): Path<i64>)
    -> Result<Json<RoomView>, ApiError>
{
    let room: RoomView = sqlx::query_as("SELECT id, name, owner_id, created_at FROM rooms WHERE id = $1")
        .bind(room_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("room not found".into()))?;

    Ok(Json(room))
}

pub async fn delete_room(auth: AuthUser, State(pool): State<PgPool>, Path(room_id): Path<i64>)
    -> Result<StatusCode, ApiError>
{
    let owner: Option<(i64,)> = sqlx::query_as("SELECT owner_id FROM rooms WHERE id = $1")
        .bind(room_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    let owner_id = owner.ok_or_else(|| ApiError::NotFound("room not found".into()))?.0;
    if owner_id != auth.user_id {
        return Err(ApiError::Forbidden("only the room owner can delete it".into()));
    }

    sqlx::query("DELETE FROM rooms WHERE id = $1")
        .bind(room_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_messages(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Path(room_id): Path<i64>,
    Query(params): Query<ListMessagesParams>,
)
    -> Result<Json<Vec<ChatMessageView>>, ApiError>
{
    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let rows: Vec<MessageRow> = sqlx::query_as(
        "SELECT m.id, m.content, m.created_at,
                u.id AS sender_id, u.username AS sender_username, u.avatar_url AS sender_avatar_url
         FROM messages m
         JOIN users u ON u.id = m.sender_id
         WHERE m.room_id = $1 AND ($2::BIGINT IS NULL OR m.id < $2)
         ORDER BY m.id DESC
         LIMIT $3",
    )
    .bind(room_id)
    .bind(params.before)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(rows.into_iter().map(ChatMessageView::from).collect()))
}
