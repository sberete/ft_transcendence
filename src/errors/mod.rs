use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal,
    // Une source externe (Archive, PeerTube...) a echoue : ce n'est pas notre bug.
    BadGateway(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    fn status_code(&self) -> StatusCode
    {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Conflict(_) => StatusCode::CONFLICT,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadGateway(_) => StatusCode::BAD_GATEWAY,
        }
    }

    fn message(&self) -> String
    {
        match self {
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::Unauthorized => "unauthorized".to_string(),
            ApiError::Forbidden(msg) => msg.clone(),
            ApiError::NotFound(msg) => msg.clone(),
            ApiError::Conflict(msg) => msg.clone(),
            ApiError::Internal => "internal server error".to_string(),
            ApiError::BadGateway(msg) => msg.clone(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response
    {
        let status = self.status_code();
        (status, Json(ErrorBody { error: self.message() })).into_response()
    }
}
