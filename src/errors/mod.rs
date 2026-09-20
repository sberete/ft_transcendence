use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    Conflict(String),
    Internal,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    fn status_code(&self)
        -> StatusCode
    {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self)
        -> String
    {
        match self {
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::Unauthorized => "unauthorized".to_string(),
            ApiError::Conflict(msg) => msg.clone(),
            ApiError::Internal => "internal server error".to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self)
        -> Response
    {
        let status = self.status_code();
        (status, Json(ErrorBody { error: self.message() })).into_response()
    }
}
