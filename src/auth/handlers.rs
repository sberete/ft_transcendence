// HTTP handlers for local (email+password) authentication: signup,
// login (with brute-force rate limiting and constant-time password
// checking), and logout. OAuth has its own handlers in `oauth/`.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::rate_limit::LoginRateLimiter;
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
    identifier: String,
    password: String,
}

#[derive(Serialize)]
pub(super) struct AuthResponse {
    token: String,
}

#[derive(sqlx::FromRow)]
struct UserCredentials {
    id: i64,
    // None for an account created via Google/GitHub.
    password_hash: Option<String>,
}

// Must match VARCHAR(32) on users.username (migration 0001): without this
// check, an over-long username would surface as a generic Postgres error
// (500) instead of a 400, since it's not a uniqueness violation.
const USERNAME_MAX_LEN: usize = 32;

// Creates a new account: validates the input, hashes the password, inserts
// the row, and immediately opens a session so the caller is logged in.
pub async fn signup(State(pool): State<PgPool>, Json(payload): Json<SignupRequest>)
    -> Result<(StatusCode, Json<AuthResponse>), ApiError>
{
    if payload.username.trim().is_empty() || payload.email.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "username and email are required".into(),
        ));
    }
    if payload.username.trim().chars().count() > USERNAME_MAX_LEN {
        return Err(ApiError::BadRequest(
            format!("username must be at most {USERNAME_MAX_LEN} characters"),
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

// Verifies credentials and opens a session. Rate-limited per IP against
// brute force, and deliberately constant-time regardless of whether the
// identifier exists (see `dummy_password_hash` below).
pub async fn login(
    State(pool): State<PgPool>,
    Extension(rate_limiter): Extension<LoginRateLimiter>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
)
    -> Result<Json<AuthResponse>, ApiError>
{
    if !rate_limiter.allow(&client_ip(&headers)) {
        return Err(ApiError::TooManyRequests("too many login attempts, try again later".into()));
    }

    let user: Option<UserCredentials> = sqlx::query_as(
        "SELECT id, password_hash FROM users WHERE email = $1 OR username = $1",
    )
    .bind(&payload.identifier)
    .fetch_optional(&pool)
    .await
    .map_err(|_| ApiError::Internal)?;

    let real_hash = user.as_ref().and_then(|u| u.password_hash.as_deref());

    // An unknown identifier or an OAuth account (no password) must take
    // exactly as long to answer as a wrong password: otherwise the response
    // time would reveal whether the account exists before anything is even
    // checked (timing-based enumeration). So we ALWAYS compare against an
    // Argon2 hash - a fixed dummy one if needed - to never do more or less
    // work than the normal case (a single verification, never a hash AND
    // a verification).
    let hash_to_check: &str = match real_hash {
        Some(hash) => hash,
        None => dummy_password_hash(),
    };
    let verified = password::verify_password(&payload.password, hash_to_check)
        .map_err(|_| ApiError::Internal)?;

    if real_hash.is_none() || !verified {
        return Err(ApiError::Unauthorized);
    }

    let token = session::create_session(&pool, user.unwrap().id)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(AuthResponse { token }))
}

// Deletes the current session; the client is expected to drop the token.
pub async fn logout(State(pool): State<PgPool>, headers: HeaderMap)
    -> Result<StatusCode, ApiError>
{
    let token = extract_bearer_token(&headers).ok_or(ApiError::Unauthorized)?;

    session::delete_session(&pool, token)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(StatusCode::NO_CONTENT)
}

// Pulls the raw token out of the `Authorization: Bearer <token>` header.
fn extract_bearer_token(headers: &HeaderMap)
    -> Option<&str>
{
    headers
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

// Fixed hash (computed once per process) to compare against when there is
// no real hash to check: an unknown identifier must never skip the Argon2
// verification step, or its shorter response time would reveal that the
// account doesn't exist.
fn dummy_password_hash() -> &'static str
{
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        password::hash_password("not-a-real-account-timing-filler")
            .expect("failed to build the dummy password hash")
    })
}

// Set by nginx (X-Real-IP, see docker/frontend/nginx.conf.template): the
// backend is never exposed directly, so this header can be trusted in
// this deployment. "unknown" groups together calls that bypass it (e.g.
// curl straight to the backend port in native dev) into a single bucket
// instead of crashing.
fn client_ip(headers: &HeaderMap)
    -> String
{
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}
