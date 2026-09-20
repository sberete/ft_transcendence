use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

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

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter)
        -> fmt::Result
    {
        match self {
            ApiError::BadRequest(msg) => write!(f, "{msg}"),
            ApiError::Unauthorized => write!(f, "unauthorized"),
            ApiError::Conflict(msg) => write!(f, "{msg}"),
            ApiError::Internal => write!(f, "internal server error"),
        }
    }
}

impl ResponseError for ApiError {
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

    fn error_response(&self)
        -> HttpResponse
    {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.to_string(),
        })
    }
}
