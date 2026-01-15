use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::json::types::JsonVehicleRoutingProblem;
use serde::Serialize;

use crate::{error::ApiError, state::AppState};

#[derive(Serialize)]
pub struct PostResponse {
    job_id: String,
}

impl IntoResponse for PostResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn post_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<JsonVehicleRoutingProblem>,
) -> Result<PostResponse, ApiError> {
    let solver_manager = &state.solver_manager;

    let problem = body.build_problem(&state.matrix_client).await?;
    let job_id = solver_manager.create_job(problem).await;
    solver_manager.start(&job_id).await;

    Ok(PostResponse { job_id })
}
