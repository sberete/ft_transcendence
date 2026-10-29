use axum::extract::{Multipart, State};
use axum::Json;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::AuthUser;
use crate::errors::ApiError;

const MAX_AVATAR_BYTES: usize = 2 * 1024 * 1024;
// Doit matcher VARCHAR(32) sur users.username (migration 0001), voir
// auth::handlers::USERNAME_MAX_LEN pour la meme contrainte a l'inscription.
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

pub async fn get_me(auth: AuthUser, State(pool): State<PgPool>)
    -> Result<Json<UserProfile>, ApiError>
{
    fetch_profile(&pool, auth.user_id).await.map(Json)
}

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

    // Ne fait jamais confiance au Content-Type multipart declare par le
    // client (n'importe qui peut le mentir) : determine le vrai type
    // depuis la signature des octets avant de choisir l'extension avec
    // laquelle le fichier sera servi statiquement (voir sniff_image_type).
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

    let avatar_url = format!("/uploads/avatars/{filename}");
    sqlx::query("UPDATE users SET avatar_url = $1 WHERE id = $2")
        .bind(&avatar_url)
        .bind(auth.user_id)
        .execute(&pool)
        .await
        .map_err(|_| ApiError::Internal)?;

    fetch_profile(&pool, auth.user_id).await.map(Json)
}

async fn fetch_profile(pool: &PgPool, user_id: i64) -> Result<UserProfile, ApiError>
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

// Identifie png/jpeg/webp par leur signature d'octets plutot que par un
// Content-Type declare (facile a mentir dans un corps multipart).
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
