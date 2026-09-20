use actix_web::{web, HttpRequest, HttpResponse};
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
struct AuthResponse {
    token: String,
}

#[derive(sqlx::FromRow)]
struct UserCredentials {
    id: i64,
    password_hash: String,
}

pub async fn signup(pool: web::Data<PgPool>, payload: web::Json<SignupRequest>) 
    -> Result<HttpResponse, ApiError> 
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
    .fetch_one(pool.get_ref())
    .await
    .map_err(|err| match &err {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ApiError::Conflict("username or email already taken".into())
        }
        _ => ApiError::Internal,
    })?;

    let token = session::create_session(pool.get_ref(), row.0)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(HttpResponse::Created().json(AuthResponse { token }))
}

pub async fn login(pool: web::Data<PgPool>, payload: web::Json<LoginRequest>) 
    -> Result<HttpResponse, ApiError> 
{
    let user: UserCredentials = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|_| ApiError::Internal)?
    .ok_or(ApiError::Unauthorized)?;

    let valid = password::verify_password(&payload.password, &user.password_hash)
        .map_err(|_| ApiError::Internal)?;
    if !valid {
        return Err(ApiError::Unauthorized);
    }

    let token = session::create_session(pool.get_ref(), user.id)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(HttpResponse::Ok().json(AuthResponse { token }))
}

pub async fn logout(pool: web::Data<PgPool>, req: HttpRequest) 
    -> Result<HttpResponse, ApiError> 
{
    let token = extract_bearer_token(&req).ok_or(ApiError::Unauthorized)?;

    session::delete_session(pool.get_ref(), token)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(HttpResponse::NoContent().finish())
}

fn extract_bearer_token(req: &HttpRequest) 
    -> Option<&str> 
{
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}
