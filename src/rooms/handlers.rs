use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;
use crate::notifications;

#[derive(Deserialize)]
pub struct CreateRoomRequest {
    name: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct RoomView {
    code: String,
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

const MAX_CODE_ATTEMPTS: u32 = 5;

fn random_code() -> String {
    use rand::Rng;
    format!("{:06}", rand::thread_rng().gen_range(0..1_000_000))
}

pub async fn create_room(auth: AuthUser, State(pool): State<PgPool>, Json(payload): Json<CreateRoomRequest>)
    -> Result<(StatusCode, Json<RoomView>), ApiError>
{
    if payload.name.trim().is_empty() {
        return Err(ApiError::BadRequest("room name is required".into()));
    }

    // Quelques tentatives en cas de collision sur le code (tres improbable
    // a 10^6 valeurs, mais une contrainte UNIQUE existe donc autant la gerer).
    for _ in 0..MAX_CODE_ATTEMPTS {
        let code = random_code();
        let result: Result<RoomView, sqlx::Error> = sqlx::query_as(
            "INSERT INTO rooms (name, owner_id, code) VALUES ($1, $2, $3)
             RETURNING code, name, owner_id, created_at",
        )
        .bind(payload.name.trim())
        .bind(auth.user_id)
        .bind(&code)
        .fetch_one(&pool)
        .await;

        match result {
            Ok(room) => {
                notify_friends_of_new_room(&pool, auth.user_id, &room).await;
                return Ok((StatusCode::CREATED, Json(room)));
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => continue,
            Err(_) => return Err(ApiError::Internal),
        }
    }

    Err(ApiError::Internal)
}

// "Un ami lance une session" (notifications, [à faire] -> fait le
// 2026-11-02) : best-effort, ne doit jamais faire echouer la creation du
// salon elle-meme.
async fn notify_friends_of_new_room(pool: &PgPool, owner_id: i64, room: &RoomView) {
    let Some((username, avatar_url)): Option<(String, Option<String>)> =
        sqlx::query_as("SELECT username, avatar_url FROM users WHERE id = $1")
            .bind(owner_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
    else {
        return;
    };

    let friend_ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT friend_id FROM friends WHERE user_id = $1 AND status = 'accepted'",
    )
    .bind(owner_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let payload = serde_json::json!({
        "from_user_id": owner_id,
        "from_username": username,
        "from_avatar_url": avatar_url,
        "room_code": room.code,
        "room_name": room.name,
    });

    for (friend_id,) in friend_ids {
        if let Err(err) = notifications::create(pool, friend_id, "room_started", payload.clone()).await {
            log::warn!("failed to create room_started notification for {friend_id}: {err}");
        }
    }
}

pub async fn get_room(_auth: AuthUser, State(pool): State<PgPool>, Path(code): Path<String>)
    -> Result<Json<RoomView>, ApiError>
{
    let room: RoomView = sqlx::query_as("SELECT code, name, owner_id, created_at FROM rooms WHERE code = $1")
        .bind(&code)
        .fetch_optional(&pool)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("room not found".into()))?;

    Ok(Json(room))
}

pub async fn list_messages(
    _auth: AuthUser,
    State(pool): State<PgPool>,
    Path(code): Path<String>,
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
         JOIN rooms r ON r.id = m.room_id
         WHERE r.code = $1 AND ($2::BIGINT IS NULL OR m.id < $2)
         ORDER BY m.id DESC
         LIMIT $3",
    )
    .bind(&code)
    .bind(params.before)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(rows.into_iter().map(ChatMessageView::from).collect()))
}
