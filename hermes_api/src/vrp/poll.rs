use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::solver::{
    accepted_solution::AcceptedSolution, score::ScoreAnalysis, solver::SolverStatus,
    working_solution::WorkingSolution,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Serialize)]
pub struct PollSolverRunning {
    solution: Option<AcceptedSolution>,
}

#[derive(Serialize)]
pub struct PollSolverCompleted {
    solution: Option<AcceptedSolution>,
}

#[derive(Serialize)]
pub enum PollResponse {
    Pending,
    Running(PollSolverRunning),
    Completed(PollSolverCompleted),
}

impl IntoResponse for PollResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn poll_handler(
    Path(job_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<PollResponse, ApiError> {
    let solver_manager = &state.solver_manager;
    if let Some(status) = solver_manager.get_status(&job_id.to_string()).await {
        match status {
            SolverStatus::Pending => Ok(PollResponse::Pending),
            SolverStatus::Running => {
                let solution = solver_manager.get_solution(&job_id.to_string()).await;
                Ok(PollResponse::Running(PollSolverRunning { solution }))
            }
            SolverStatus::Completed => {
                let solution = solver_manager.get_solution(&job_id.to_string()).await;
                Ok(PollResponse::Completed(PollSolverCompleted { solution }))
            }
        }
    } else {
        Err(ApiError::NotFound(job_id.to_string()))
    }
}
