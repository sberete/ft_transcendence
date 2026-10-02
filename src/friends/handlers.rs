use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

const ONLINE_THRESHOLD_SECONDS: &str = "60 seconds";

#[derive(Deserialize)]
pub struct AddFriendRequest {
    friend_id: i64,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct FriendView {
    id: i64,
    username: String,
    avatar_url: Option<String>,
    online: bool,
}

pub async fn list_friends(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<Vec<FriendView>>, ApiError>
{
    let friends: Vec<FriendView> = sqlx::query_as(&format!(
        "SELECT u.id, u.username, u.avatar_url,
                (u.last_seen_at > now() - interval '{ONLINE_THRESHOLD_SECONDS}') AS online
         FROM friends f
         JOIN users u ON u.id = f.friend_id
         WHERE f.user_id = $1 AND f.status = 'accepted'
         ORDER BY u.username"
    ))
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(friends))
}

pub async fn add_friend(auth: AuthUser, State(pool): State<PgPool>, Json(payload): Json<AddFriendRequest>)
    -> Result<Json<FriendView>, ApiError>
{
    if payload.friend_id == auth.user_id {
        return Err(ApiError::BadRequest("cannot add yourself as a friend".into()));
    }

    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(payload.friend_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    if exists.is_none() {
        return Err(ApiError::NotFound("user not found".into()));
    }

    let insert = sqlx::query(
        "INSERT INTO friends (user_id, friend_id, status) VALUES ($1, $2, 'accepted'), ($2, $1, 'accepted')",
    )
    .bind(auth.user_id)
    .bind(payload.friend_id)
    .execute(&pool)
    .await;

    match insert {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return Err(ApiError::Conflict("already friends".into()));
        }
        Err(_) => return Err(ApiError::Internal),
    }

    fetch_friend_view(&pool, payload.friend_id).await
}

pub async fn remove_friend(auth: AuthUser, State(pool): State<PgPool>, Path(friend_id): Path<i64>)
    -> Result<StatusCode, ApiError>
{
    sqlx::query(
        "DELETE FROM friends WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
    )
    .bind(auth.user_id)
    .bind(friend_id)
    .execute(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_friend_view(pool: &PgPool, id: i64)
    -> Result<Json<FriendView>, ApiError>
{
    let view: FriendView = sqlx::query_as(&format!(
        "SELECT id, username, avatar_url,
                (last_seen_at > now() - interval '{ONLINE_THRESHOLD_SECONDS}') AS online
         FROM users WHERE id = $1"
    ))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(view))
}
