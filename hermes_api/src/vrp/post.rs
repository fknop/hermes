use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::json::types::JsonVehicleRoutingProblem;
use serde::Serialize;
use uuid::Uuid;

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

    let job_id = Uuid::new_v4().to_string();

    let vrp = body.build_problem(&state.matrix_client).await?;

    solver_manager.solve(job_id.clone(), vrp).await;

    Ok(PostResponse { job_id })
}
