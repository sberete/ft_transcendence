use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::{password, session};
use crate::errors::ApiError;

#[derive(Deserialize)]
pub struct SignupRequest {
    username: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub(super) struct AuthResponse {
    token: String,
}

#[derive(sqlx::FromRow)]
struct UserCredentials {
    id: i64,
    password_hash: String,
}

pub async fn signup(State(pool): State<PgPool>, Json(payload): Json<SignupRequest>)
    -> Result<(StatusCode, Json<AuthResponse>), ApiError>
{
    if payload.username.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "username and email are required".into(),
        ));
    }
    if payload.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters".into(),
        ));
    }

    let password_hash = password::hash_password(&payload.password).map_err(|_| ApiError::Internal)?;

    let row: (i64,) = sqlx::query_as(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ApiError::Conflict("username or email already taken".into())
        }
        _ => ApiError::Internal,
    })?;

    let token = session::create_session(&pool, row.0)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(AuthResponse { token })))
}

pub async fn login(State(pool): State<PgPool>, Json(payload): Json<LoginRequest>)
    -> Result<Json<AuthResponse>, ApiError>
{
    let user: UserCredentials = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await
    .map_err(|_| ApiError::Internal)?
    .ok_or(ApiError::Unauthorized)?;

    let valid = password::verify_password(&payload.password, &user.password_hash)
        .map_err(|_| ApiError::Internal)?;
    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let token = session::create_session(&pool, user.id)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(AuthResponse { token }))
}

pub async fn logout(State(pool): State<PgPool>, headers: HeaderMap)
    -> Result<StatusCode, ApiError>
{
    let token = extract_bearer_token(&headers).ok_or(ApiError::Unauthorized)?;

    session::delete_session(&pool, token)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn extract_bearer_token(headers: &HeaderMap)
    -> Option<&str>
{
    headers
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}
