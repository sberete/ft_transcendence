// HTTP handlers for the friends system: send/accept a request, list
// friends and pending requests, remove a friend. Each mutation also
// fires a best-effort notification to the other party.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;
use crate::notifications;

const ONLINE_THRESHOLD_SECONDS: &str = "60 seconds";

// Used to denormalize who triggered a notification (see
// notifications::create) without repeating the same query in every handler.
async fn user_ref(pool: &PgPool, user_id: i64) -> Option<(String, Option<String>)> {
    sqlx::query_as("SELECT username, avatar_url FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

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

// Lists this user's accepted friends, with their online status.
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

// Lists incoming friend requests still pending this user's decision.
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

// Creates a pending friend request by username, after checking it isn't
// a self-request or a duplicate of an existing relationship, then
// notifies the target user.
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

    if let Some((username, avatar_url)) = user_ref(&pool, auth.user_id).await {
        let payload = serde_json::json!({
            "from_user_id": auth.user_id,
            "from_username": username,
            "from_avatar_url": avatar_url,
        });
        if let Err(err) = notifications::create(&pool, friend_id, "friend_request", payload).await {
            log::warn!("failed to create friend_request notification: {err}");
        }
    }

    Ok(StatusCode::CREATED)
}

// Accepts a pending request from `from_user_id`: flips the original row
// to 'accepted' and inserts the mirrored row so the relationship is
// queryable from either side, then notifies the requester.
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

    if let Some((username, avatar_url)) = user_ref(&pool, auth.user_id).await {
        let payload = serde_json::json!({
            "from_user_id": auth.user_id,
            "from_username": username,
            "from_avatar_url": avatar_url,
        });
        if let Err(err) = notifications::create(&pool, from_user_id, "friend_accepted", payload).await {
            log::warn!("failed to create friend_accepted notification: {err}");
        }
    }

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

// Removes the friendship in both directions (whichever side initiated
// it originally) and notifies the other party.
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

    if let Some((username, avatar_url)) = user_ref(&pool, auth.user_id).await {
        let payload = serde_json::json!({
            "from_user_id": auth.user_id,
            "from_username": username,
            "from_avatar_url": avatar_url,
        });
        if let Err(err) = notifications::create(&pool, friend_id, "friend_removed", payload).await {
            log::warn!("failed to create friend_removed notification: {err}");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
