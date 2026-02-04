use std::sync::Arc;

use axum::{Json, extract::State};
use hermes_optimizer::solver::solver::SolverStatus;
use jiff::Timestamp;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{error::ApiError, pagination::PaginatedResponse, state::AppState};

#[derive(Serialize, JsonSchema)]
pub struct VehicleRoutingJob {
    pub job_id: String,
    pub status: SolverStatus,
    pub created_at: Timestamp,
}

pub async fn jobs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<PaginatedResponse<VehicleRoutingJob>>, ApiError> {
    let solver_manager = &state.solver_manager;
    let solvers = solver_manager.list_solvers().await;

    let mut jobs: Vec<VehicleRoutingJob> = solvers
        .into_iter()
        .map(|(job_id, solver)| VehicleRoutingJob {
            job_id,
            status: solver.status(),
            created_at: solver.created_at(),
        })
        .collect();

    jobs.sort_by(|job1, job2| job2.created_at.cmp(&job1.created_at));

    Ok(Json(PaginatedResponse {
        page: 1,
        per_page: jobs.len(),
        total: jobs.len(),
        data: jobs,
        total_pages: 1,
    }))
}
