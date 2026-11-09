// Session tokens backed by the database: random opaque tokens are
// handed to clients, but only their SHA-256 hash is ever stored, so a
// stolen database dump doesn't hand out usable tokens.

use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

const SESSION_DURATION_HOURS: i64 = 24 * 7;

// Generates a new random token, stores its hash with an expiry, and
// returns the raw token for the client to use as a Bearer credential.
pub async fn create_session(pool: &PgPool, user_id: i64)
    -> Result<String, sqlx::Error>
{
    let mut raw = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut raw);
    let token = hex::encode(raw);
    let expires_at = Utc::now() + Duration::hours(SESSION_DURATION_HOURS);

    sqlx::query("INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(hash_token(&token))
        .bind(expires_at)
        .execute(pool)
        .await?;

    Ok(token)
}

// Removes a session row, effectively logging that token out.
pub async fn delete_session(pool: &PgPool, token: &str)
    -> Result<(), sqlx::Error>
{
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(hash_token(token))
        .execute(pool)
        .await?;

    Ok(())
}

// Looks up the user behind a token, if it exists and hasn't expired.
pub async fn validate_session(pool: &PgPool, token: &str)
    -> Result<Option<i64>, sqlx::Error>
{
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT user_id FROM sessions WHERE token_hash = $1 AND expires_at > now()",
    )
    .bind(hash_token(token))
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0))
}

// Deterministic hash used to look up a token without storing it in plain text.
fn hash_token(token: &str)
    -> String
{
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
