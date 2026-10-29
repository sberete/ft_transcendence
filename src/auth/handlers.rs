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
    // None pour un compte cree via Google/GitHub.
    password_hash: Option<String>,
}

// Doit matcher VARCHAR(32) sur users.username (migration 0001) : sans ce
// controle, un pseudo trop long remonte comme une erreur Postgres generique
// (500) au lieu d'un 400, puisque ce n'est pas une violation d'unicite.
const USERNAME_MAX_LEN: usize = 32;

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

    // Un identifiant inconnu ou un compte OAuth (sans mot de passe) doit
    // prendre le meme temps a repondre qu'un mauvais mot de passe : sinon
    // la duree de reponse revele si le compte existe avant meme de
    // verifier quoi que ce soit (enumeration par le temps). On compare
    // donc TOUJOURS contre un hash Argon2 - un faux fixe si besoin - pour
    // ne jamais faire plus ou moins de travail que le cas normal (une
    // seule verification, jamais un hash ET une verification).
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

// Hash fixe (calcule une seule fois par processus) contre lequel comparer
// quand il n'y a pas de vrai hash a verifier : un identifiant inconnu ne
// doit jamais eviter l'etape de verification Argon2, sinon sa duree de
// reponse plus courte revele que le compte n'existe pas.
fn dummy_password_hash() -> &'static str
{
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        password::hash_password("not-a-real-account-timing-filler")
            .expect("failed to build the dummy password hash")
    })
}

// Pose par nginx (X-Real-IP, voir docker/frontend/nginx.conf.template) :
// le backend n'est jamais expose directement, donc cet en-tete est fiable
// dans ce deploiement. "unknown" regroupe les appels qui le contournent
// (ex: curl direct sur le port du backend en dev natif) dans un seul
// compteur plutot que de planter.
fn client_ip(headers: &HeaderMap)
    -> String
{
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}
