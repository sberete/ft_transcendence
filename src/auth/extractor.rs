use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use sqlx::PgPool;

use super::session;
use crate::errors::ApiError;

pub struct AuthUser {
    pub user_id: i64,
}

impl FromRequestParts<PgPool> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, pool: &PgPool)
        -> Result<Self, Self::Rejection>
    {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthorized)?;

        let user_id = session::validate_session(pool, token)
            .await
            .map_err(|_| ApiError::Internal)?
            .ok_or(ApiError::Unauthorized)?;

        // Best-effort : sert de signal de presence pour le statut "en ligne".
        // Une erreur ici ne doit jamais faire echouer la requete elle-meme.
        if let Err(err) = sqlx::query("UPDATE users SET last_seen_at = now() WHERE id = $1")
            .bind(user_id)
            .execute(pool)
            .await
        {
            log::warn!("failed to update last_seen_at for user {user_id}: {err}");
        }

        Ok(AuthUser { user_id })
    }
}
