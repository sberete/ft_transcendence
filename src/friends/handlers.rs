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
    username: String,
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

pub async fn list_requests(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<Vec<FriendView>>, ApiError>
{
    let requests: Vec<FriendView> = sqlx::query_as(&format!(
        "SELECT u.id, u.username, u.avatar_url,
                (u.last_seen_at > now() - interval '{ONLINE_THRESHOLD_SECONDS}') AS online
         FROM friends f
         JOIN users u ON u.id = f.user_id
         WHERE f.friend_id = $1 AND f.status = 'pending'
         ORDER BY u.username"
    ))
    .bind(auth.user_id)
    .fetch_all(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(requests))
}

pub async fn send_request(auth: AuthUser, State(pool): State<PgPool>, Json(payload): Json<AddFriendRequest>)
    -> Result<StatusCode, ApiError>
{
    let target: Option<(i64,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
        .bind(&payload.username)
        .fetch_optional(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    let friend_id = target.ok_or_else(|| ApiError::NotFound("user not found".into()))?.0;

    if friend_id == auth.user_id {
        return Err(ApiError::BadRequest("cannot add yourself as a friend".into()));
    }

    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM friends WHERE (user_id = $1 AND friend_id = $2) OR (user_id = $2 AND friend_id = $1)",
    )
    .bind(auth.user_id)
    .bind(friend_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    if existing.is_some() {
        return Err(ApiError::Conflict(
            "already friends or a request is already pending".into(),
        ));
    }

    sqlx::query("INSERT INTO friends (user_id, friend_id, status) VALUES ($1, $2, 'pending')")
        .bind(auth.user_id)
        .bind(friend_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::CREATED)
}

pub async fn accept_request(auth: AuthUser, State(pool): State<PgPool>, Path(from_user_id): Path<i64>)
    -> Result<Json<FriendView>, ApiError>
{
    let mut tx = pool.begin().await.map_err(|_| ApiError::Internal)?;

    let updated = sqlx::query(
        "UPDATE friends SET status = 'accepted' WHERE user_id = $1 AND friend_id = $2 AND status = 'pending'",
    )
    .bind(from_user_id)
    .bind(auth.user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::Internal)?;

    if updated.rows_affected() == 0 {
        return Err(ApiError::NotFound("no pending request from this user".into()));
    }

    sqlx::query(
        "INSERT INTO friends (user_id, friend_id, status) VALUES ($1, $2, 'accepted')
         ON CONFLICT (user_id, friend_id) DO UPDATE SET status = 'accepted'",
    )
    .bind(auth.user_id)
    .bind(from_user_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| ApiError::Internal)?;

    tx.commit().await.map_err(|_| ApiError::Internal)?;

    let view: FriendView = sqlx::query_as(&format!(
        "SELECT id, username, avatar_url,
                (last_seen_at > now() - interval '{ONLINE_THRESHOLD_SECONDS}') AS online
         FROM users WHERE id = $1"
    ))
    .bind(from_user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    Ok(Json(view))
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
