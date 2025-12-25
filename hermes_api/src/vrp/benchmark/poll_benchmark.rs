use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use hermes_optimizer_core::solver::{
    accepted_solution::AcceptedSolution, solver::SolverStatus, statistics::SearchStatistics,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

use super::benchmark_solution::{
    BenchmarkServiceActivity, BenchmarkSolution, BenchmarkSolutionActivity, BenchmarkSolutionRoute,
};

#[derive(Serialize)]
pub struct PollSolverRunning {
    solution: Option<BenchmarkSolution>,
    statistics: Option<Arc<SearchStatistics>>,
}

#[derive(Serialize)]
pub struct PollSolverCompleted {
    solution: Option<BenchmarkSolution>,
    statistics: Option<Arc<SearchStatistics>>,
}

#[derive(Serialize)]
#[serde(tag = "status")]
pub enum PollBenchmarkResponse {
    Pending,
    Running(PollSolverRunning),
    Completed(PollSolverCompleted),
}

fn transform_solution(accepted_solution: &AcceptedSolution) -> BenchmarkSolution {
    let problem = accepted_solution.solution.problem();
    let routes: Vec<BenchmarkSolutionRoute> = accepted_solution
        .solution
        .non_empty_routes_iter()
        .map(|route| {
            let mut activities: Vec<BenchmarkSolutionActivity> = vec![];

            activities.extend(route.activity_ids().iter().map(|activity| {
                BenchmarkSolutionActivity::Service(BenchmarkServiceActivity {
                    service_id: activity.index(),
                })
            }));

            BenchmarkSolutionRoute {
                distance: route.distance(problem),
                total_demand: route.total_initial_load().clone(),
                vehicle_id: route.vehicle_id(),
                waiting_duration: route.total_waiting_duration(),
                activities,
                vehicle_max_load: route.max_load(problem),
            }
        })
        .collect();

    BenchmarkSolution {
        score: accepted_solution.score,
        score_analysis: accepted_solution.score_analysis.clone(),
        distance: routes.iter().fold(0.0, |acc, route| acc + route.distance),
        routes,
    }
}

impl IntoResponse for PollBenchmarkResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub async fn poll_handler(
    Path(job_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<PollBenchmarkResponse, ApiError> {
    let solver_manager = &state.solver_manager;
    if let Some(status) = solver_manager.get_status(&job_id.to_string()).await {
        match status {
            SolverStatus::Pending => Ok(PollBenchmarkResponse::Pending),
            SolverStatus::Running => {
                let solution = solver_manager.get_solution(&job_id.to_string()).await;
                let statistics = solver_manager.get_statistics(&job_id.to_string()).await;

                Ok(PollBenchmarkResponse::Running(PollSolverRunning {
                    solution: solution.map(|solution| transform_solution(&solution)),
                    statistics,
                }))
            }
            SolverStatus::Completed => {
                let solution = solver_manager.get_solution(&job_id.to_string()).await;
                let statistics = solver_manager.get_statistics(&job_id.to_string()).await;
                Ok(PollBenchmarkResponse::Completed(PollSolverCompleted {
                    solution: solution.map(|solution| transform_solution(&solution)),
                    statistics,
                }))
            }
        }
    } else {
        Err(ApiError::NotFound(job_id.to_string()))
    }
}
