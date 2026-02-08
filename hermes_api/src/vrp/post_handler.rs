use std::sync::Arc;

use axum::{Json, extract::State};
use hermes_optimizer::json::types::JsonVehicleRoutingProblem;
use schemars::JsonSchema;
use serde::Serialize;
use tracing::info;

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

    Ok(Json(PostResponse { job_id }))
}
