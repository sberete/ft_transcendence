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

        Ok(AuthUser { user_id })
    }
}
