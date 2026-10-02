use axum::extract::{Multipart, State};
use axum::Json;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;

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

pub async fn get_me(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<UserProfile>, ApiError>
{
    fetch_profile(&pool, auth.user_id).await.map(Json)
}

pub async fn update_me(
    auth: AuthUser,
    State(pool): State<PgPool>,
    Json(payload): Json<UpdateProfileRequest>,
)
    -> Result<Json<UserProfile>, ApiError>
{
    if let Some(username) = &payload.username {
        if username.trim().is_empty() {
            return Err(ApiError::BadRequest("username cannot be empty".into()));
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

pub async fn upload_avatar(auth: AuthUser, State(pool): State<PgPool>, mut multipart: Multipart)
    -> Result<Json<UserProfile>, ApiError>
{
    let field = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::BadRequest("invalid multipart body".into()))?
        .ok_or(ApiError::BadRequest("missing file field".into()))?;

    let extension = match field.content_type() {
        Some("image/png") => "png",
        Some("image/jpeg") => "jpg",
        Some("image/webp") => "webp",
        _ => return Err(ApiError::BadRequest("avatar must be png, jpeg or webp".into())),
    };

    let data = field.bytes().await.map_err(|_| ApiError::Internal)?;
    if data.len() > MAX_AVATAR_BYTES {
        return Err(ApiError::BadRequest("avatar must be under 2MB".into()));
    }

    let filename = format!("{}.{extension}", random_filename_stem());
    let dir = "uploads/avatars";
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|_| ApiError::Internal)?;
    tokio::fs::write(format!("{dir}/{filename}"), &data)
        .await
        .map_err(|_| ApiError::Internal)?;

    let avatar_url = format!("/uploads/avatars/{filename}");
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(&avatar_url)
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    fetch_profile(&pool, auth.user_id).await.map(Json)
}

async fn fetch_profile(pool: &PgPool, user_id: i64)
    -> Result<UserProfile, ApiError>
{
    sqlx::query_as("SELECT id, username, email, avatar_url FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|_| ApiError::Internal)
}

fn random_filename_stem() -> String
{
    let mut raw = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut raw);
    hex::encode(raw)
}
