use aide::{OperationOutput, generate::GenContext, openapi::Operation};

use axum::{Json, http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
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
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
        }
    }
}

impl OperationOutput for ApiError {
    type Inner = Json<String>;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        <Json<String>>::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Vec<(Option<aide::openapi::StatusCode>, aide::openapi::Response)> {
        // println!("{:?}", operation);
        if let Some(res) = Self::operation_response(ctx, operation) {
            Vec::from([
                // (Some(aide::openapi::StatusCode::Code(200)), res.clone()),
                (
                    Some(aide::openapi::StatusCode::Code(404)),
                    aide::openapi::Response {
                        description: "Not found".into(),
                        ..res.clone()
                    },
                ),
                (
                    Some(aide::openapi::StatusCode::Code(500)),
                    aide::openapi::Response {
                        description: "Internal server error".into(),
                        ..res.clone()
                    },
                ),
            ])
        } else {
            Vec::new()
        }
    }
}
