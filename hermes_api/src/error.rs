use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub enum ApiError {
    BadRequest(String),
    InternalServerError(String),
    NotFound(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        ApiError::InternalServerError(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::InternalServerError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
        }
    }
}
