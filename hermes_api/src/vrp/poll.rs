use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use geo::{Coord, Simplify};
use geojson::{Feature, Geometry};
use hermes_optimizer::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        accepted_solution::AcceptedSolution, solution::route::WorkingSolutionRoute,
        solver::SolverStatus,
    },
};
use hermes_routing::{
    geopoint::GeoPoint,
    hermes::Hermes,
    routing::routing_request::{RoutingAlgorithm, RoutingRequest, RoutingRequestOptions},
};
use jiff::SignedDuration;
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

use super::api_solution::{
    ApiEndActivity, ApiServiceActivity, ApiSolution, ApiSolutionActivity, ApiSolutionRoute,
    ApiStartActivity,
};

#[derive(Serialize)]
pub struct PollSolverRunning {
    solution: Option<ApiSolution>,
}

#[derive(Serialize)]
pub struct PollSolverCompleted {
    solution: Option<ApiSolution>,
}

#[derive(Serialize)]
#[serde(tag = "status")]
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

fn compute_polyline(
    problem: &VehicleRoutingProblem,
    route: &WorkingSolutionRoute,
    hermes: &Hermes,
) -> Feature {
    let location_ids = route.compute_location_ids(problem);

    let mut points: Vec<Coord<f64>> = vec![];
    for (index, &location_id) in location_ids.iter().enumerate() {
        if index == location_ids.len() - 1 {
            continue;
        }

        let next_location_id = location_ids[index + 1];

        let location = problem.location(location_id);
        let next_location = problem.location(next_location_id);

        let result = hermes
            .route(RoutingRequest {
                start: GeoPoint::new(location.lon(), location.lat()),
                end: GeoPoint::new(next_location.lon(), next_location.lat()),
                profile: String::from("car"),
                options: Some(RoutingRequestOptions {
                    algorithm: Some(RoutingAlgorithm::ContractionHierarchies),
                    include_debug_info: None,
                }),
            })
            .unwrap();

        points.extend(result.path.legs().iter().flat_map(|leg| {
            leg.points().iter().map(|point| geo::Coord {
                x: point.lon(),
                y: point.lat(),
            })
        }));
    }

    let geometry = geo::LineString::new(points).simplify(&0.0001);

    Feature {
        geometry: Some(Geometry::from(&geometry)),
        ..Default::default()
    }
}

fn transform_solution(accepted_solution: &AcceptedSolution, hermes: &Hermes) -> ApiSolution {
    let problem = accepted_solution.solution.problem();
    let routes: Vec<ApiSolutionRoute> = accepted_solution
        .solution
        .non_empty_routes_iter()
        .map(|route| {
            let vehicle = problem.vehicle(route.vehicle_id());
            let mut activities: Vec<ApiSolutionActivity> = vec![];
            if route.has_start(problem) {
                activities.push(ApiSolutionActivity::Start(ApiStartActivity {
                    arrival_time: route.start(problem),
                    departure_time: route.start(problem) + vehicle.depot_duration(),
                }));
            }

            activities.extend(route.activities_iter().map(|activity| {
                ApiSolutionActivity::Service(ApiServiceActivity {
                    service_id: activity.activity_id().job_id(),
                    arrival_time: activity.arrival_time(),
                    departure_time: activity.departure_time(),
                    waiting_duration: activity.waiting_duration(),
                })
            }));

            if route.has_end(problem) {
                activities.push(ApiSolutionActivity::End(ApiEndActivity {
                    arrival_time: route.end(problem) - vehicle.end_depot_duration(),
                    departure_time: route.end(problem),
                }));
            }

            ApiSolutionRoute {
                distance: route.distance(problem),
                duration: route.duration(problem),
                transport_duration: route.transport_duration(problem),
                total_demand: route.total_initial_load().clone(),
                vehicle_id: route.vehicle_id(),
                waiting_duration: route.total_waiting_duration(),
                activities,
                polyline: compute_polyline(problem, route, hermes),
                vehicle_max_load: route.max_load(problem),
            }
        })
        .collect();

    ApiSolution {
        score: accepted_solution.score,
        score_analysis: accepted_solution.score_analysis.clone(),
        duration: routes
            .iter()
            .fold(SignedDuration::ZERO, |acc, route| acc + route.duration),
        routes,
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
                Ok(PollResponse::Running(PollSolverRunning {
                    solution: solution.map(|solution| transform_solution(&solution, &state.hermes)),
                }))
            }
            SolverStatus::Completed => {
                let solution = solver_manager.get_solution(&job_id.to_string()).await;
                Ok(PollResponse::Completed(PollSolverCompleted {
                    solution: solution.map(|solution| transform_solution(&solution, &state.hermes)),
                }))
            }
        }
    } else {
        println!("NOT FOUND?");
        Err(ApiError::NotFound(job_id.to_string()))
    }
}
