use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::json::types::JsonVehicleRoutingProblem;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{error::ApiError, state::AppState};

#[derive(Serialize, JsonSchema)]
pub struct PostResponse {
    job_id: String,
}

pub async fn post_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<JsonVehicleRoutingProblem>,
) -> Result<Json<PostResponse>, ApiError> {
    let solver_manager = &state.solver_manager;

    let problem = body.build_problem(&state.matrix_client).await?;
    let job_id = solver_manager.create_job(problem).await;

    solver_manager.start(&job_id).await;

    Ok(Json(PostResponse { job_id }))
}
