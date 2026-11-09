// HTTP handlers for the current user's own profile and avatar: read,
// rename, upload a new avatar (validated by real file signature, not
// the declared Content-Type), and delete it, with disk cleanup on both
// replace and delete.

use axum::extract::{Multipart, State};
use axum::Json;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;
// Must match VARCHAR(32) on users.username (migration 0001); see
// auth::handlers::USERNAME_MAX_LEN for the same constraint at signup.
const USERNAME_MAX_LEN: usize = 32;

#[derive(Serialize, sqlx::FromRow)]
pub struct UserProfile {
    id: i64,
    username: String,
    email: String,
    avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    username: Option<String>,
}

// Returns the caller's own profile.
pub async fn get_me(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<UserProfile>, ApiError>
{
    fetch_profile(&pool, auth.user_id).await.map(Json)
}

// Updates the caller's username (when provided), after validating its
// length, then returns the refreshed profile.
pub async fn update_me(auth: AuthUser, State(pool): State<PgPool>,
    Json(payload): Json<UpdateProfileRequest>)
        -> Result<Json<UserProfile>, ApiError>
{
    if let Some(username) = &payload.username {
        if username.trim().is_empty() {
            return Err(ApiError::BadRequest("username cannot be empty".into()));
        }
        if username.trim().chars().count() > USERNAME_MAX_LEN {
            return Err(ApiError::BadRequest(
                format!("username must be at most {USERNAME_MAX_LEN} characters"),
            ));
        }

        sqlx::query("UPDATE users SET username = $1 WHERE id = $2")
            .bind(username)
            .bind(auth.user_id)
            .execute(&pool)
            .await
            .map_err(|err| match &err {
                sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                    ApiError::Conflict("username already taken".into())
                }
                _ => ApiError::Internal,
            })?;
    }

    fetch_profile(&pool, auth.user_id).await.map(Json)
}

// Accepts a multipart image upload, validates its real type and size,
// writes it to disk under a random name, points the profile at it, and
// cleans up whatever avatar file it replaced.
pub async fn upload_avatar(auth: AuthUser, State(pool): State<PgPool>, mut multipart: Multipart) -> Result<Json<UserProfile>, ApiError>
{
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("invalid multipart body".into()))?
        .ok_or(ApiError::BadRequest("missing file field".into()))?;

    let data = field.bytes().await.map_err(|_| ApiError::Internal)?;
    if data.len() > MAX_AVATAR_BYTES {
        return Err(ApiError::BadRequest("avatar must be under 2MB".into()));
    }

    // Never trusts the multipart Content-Type declared by the client
    // (anyone can lie about it): determines the real type from the byte
    // signature before picking the extension the file will be served
    // under statically (see sniff_image_type).
    let extension = sniff_image_type(&data)
        .ok_or_else(|| ApiError::BadRequest("avatar must be a real png, jpeg or webp file".into()))?;

    let filename = format!("{}.{extension}", random_filename_stem());
    let dir = "uploads/avatars";
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| ApiError::Internal)?;
    tokio::fs::write(format!("{dir}/{filename}"), &data)
        .await
        .map_err(|_| ApiError::Internal)?;

    // Fetches the previous avatar BEFORE overwriting the column: once the
    // new one is saved, there's no trace left of which file to clean up.
    let previous_avatar_url = current_avatar_url(&pool, auth.user_id).await?;

    let avatar_url = format!("/uploads/avatars/{filename}");
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(&avatar_url)
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    if let Some(previous) = previous_avatar_url {
        remove_avatar_file(&previous).await;
    }

    fetch_profile(&pool, auth.user_id).await.map(Json)
}

// Clears the caller's avatar and removes the underlying file from disk.
pub async fn delete_avatar(auth: AuthUser, State(pool): State<PgPool>) -> Result<Json<UserProfile>, ApiError>
{
    let previous_avatar_url = current_avatar_url(&pool, auth.user_id).await?;

    sqlx::query("UPDATE users SET avatar_url = NULL WHERE id = $1")
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    if let Some(previous) = previous_avatar_url {
        remove_avatar_file(&previous).await;
    }

    fetch_profile(&pool, auth.user_id).await.map(Json)
}

// Loads one user's public profile fields.
async fn fetch_profile(pool: &PgPool, user_id: i64) -> Result<UserProfile, ApiError>
{
    sqlx::query_as("SELECT id, username, email, avatar_url FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

// Reads just the current avatar_url, used before overwriting or clearing it.
async fn current_avatar_url(pool: &PgPool, user_id: i64) -> Result<Option<String>, ApiError>
{
    let row: (Option<String>,) = sqlx::query_as("SELECT avatar_url FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::Internal)?;
    Ok(row.0)
}

// Best-effort: an avatar_url only ever points to a file we wrote
// ourselves under uploads/avatars/ (never an external URL, see the
// manual's Security section). If the removal fails, the request still
// doesn't fail because of it - an orphaned file breaks nothing, unlike
// a profile update that would otherwise look like it failed.
async fn remove_avatar_file(avatar_url: &str)
{
    let path = avatar_url.trim_start_matches('/');
    if let Err(err) = tokio::fs::remove_file(path).await {
        log::warn!("failed to remove old avatar file {path}: {err}");
    }
}

// Generates a random filename stem for a newly uploaded avatar.
fn random_filename_stem() -> String
{
    let mut raw = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut raw);
    hex::encode(raw)
}

// Identifies png/jpeg/webp from their byte signature rather than a
// declared Content-Type (easy to lie about in a multipart body).
fn sniff_image_type(data: &[u8]) -> Option<&'static str>
{
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("png")
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("jpg")
    } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}
