use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use geo::{Coord, Simplify};
use geojson::{Feature, Geometry};
use hermes_optimizer::{
    json::types::{FromProblem as _, JsonLocation, JsonService, JsonVehicle},
    problem::{job::Job, meters::Meters, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        accepted_solution::AcceptedSolution, alns_weights::AlnsWeights,
        recreate::recreate_strategy::RecreateStrategy, ruin::ruin_strategy::RuinStrategy,
        solution::route::WorkingSolutionRoute, solver::SolverStatus,
        statistics::AggregatedStatistics,
    },
};
use hermes_routing::{
    geopoint::GeoPoint,
    hermes::Hermes,
    routing::routing_request::{RoutingAlgorithm, RoutingRequest, RoutingRequestOptions},
};
use jiff::SignedDuration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

use super::api_solution::{
    ApiEndActivity, ApiServiceActivity, ApiSolution, ApiSolutionActivity, ApiSolutionRoute,
    ApiStartActivity,
};

#[derive(Serialize, JsonSchema)]
struct OperatorWeights {
    ruin: AlnsWeights<RuinStrategy>,
    recreate: AlnsWeights<RecreateStrategy>,
}

#[derive(Serialize, JsonSchema)]
pub struct PollSolverRunning {
    solution: Option<ApiSolution>,
    statistics: AggregatedStatistics,
    weights: OperatorWeights,
}

#[derive(Serialize, JsonSchema)]
pub struct PollSolverCompleted {
    solution: Option<ApiSolution>,
    statistics: AggregatedStatistics,
    weights: OperatorWeights,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "status")]
pub enum PollResponse {
    Pending,
    Running(PollSolverRunning),
    Completed(PollSolverCompleted),
}

#[derive(Deserialize, JsonSchema)]
pub struct JobPath {
    pub job_id: Uuid,
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

fn transform_solution(
    accepted_solution: &AcceptedSolution,
    hermes: &Hermes,
    with_geojson: bool,
) -> ApiSolution {
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
                    id: problem
                        .job(activity.activity_id().job_id())
                        .external_id()
                        .to_owned(),
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
                vehicle_id: route.vehicle(problem).external_id().to_owned(),
                waiting_duration: route.total_waiting_duration(),
                activities,
                polyline: if with_geojson {
                    compute_polyline(problem, route, hermes)
                } else {
                    Feature::default()
                },
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
        distance: routes
            .iter()
            .fold(Meters::ZERO, |acc, route| acc + route.distance),
        routes,
        unassigned_jobs: accepted_solution
            .solution
            .unassigned_jobs()
            .iter()
            .map(|job_id| problem.job(*job_id).external_id().to_owned())
            .collect::<Vec<_>>(),
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct PollQuery {
    geojson: Option<bool>,
}

pub async fn poll_handler(
    Path(path): Path<JobPath>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<PollQuery>,
) -> Result<Json<PollResponse>, ApiError> {
    let solver = state
        .solver_manager
        .solver(&path.job_id.to_string())
        .await
        .ok_or(ApiError::NotFound(path.job_id.to_string()))?;

    match solver.status() {
        SolverStatus::Pending => Ok(Json(PollResponse::Pending)),
        SolverStatus::Running => {
            let solution = solver.current_best_solution().map(|solution| {
                transform_solution(&solution, &state.hermes, query.geojson.unwrap_or(true))
            });
            let statistics = solver.statistics().aggregate();
            let weights = solver.weights();
            Ok(Json(PollResponse::Running(PollSolverRunning {
                solution,
                statistics,
                weights: OperatorWeights {
                    ruin: weights.0,
                    recreate: weights.1,
                },
            })))
        }
        SolverStatus::Completed => {
            let solution = solver.current_best_solution().map(|solution| {
                transform_solution(&solution, &state.hermes, query.geojson.unwrap_or(true))
            });
            let statistics = solver.statistics().aggregate();
            let weights = solver.weights();
            Ok(Json(PollResponse::Completed(PollSolverCompleted {
                solution,
                statistics,
                weights: OperatorWeights {
                    ruin: weights.0,
                    recreate: weights.1,
                },
            })))
        }
    }
}

pub async fn start_handler(
    Path(path): Path<JobPath>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<bool>, ApiError> {
    state.solver_manager.start(&path.job_id.to_string()).await;

    if true {
        Ok(Json(true))
    } else {
        Err(ApiError::NotFound(path.job_id.to_string()))
    }
}

pub async fn stop_handler(
    Path(path): Path<JobPath>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<bool>, ApiError> {
    let result = state.solver_manager.stop(&path.job_id.to_string()).await;

    if result {
        Ok(Json(true))
    } else {
        Err(ApiError::NotFound(path.job_id.to_string()))
    }
}

#[derive(Serialize, JsonSchema)]
pub struct VehicleRoutingJobInput {
    pub id: String,
    pub locations: Vec<JsonLocation>,
    pub vehicles: Vec<JsonVehicle>,
    pub services: Vec<JsonService>,
}

impl From<&VehicleRoutingProblem> for VehicleRoutingJobInput {
    fn from(problem: &VehicleRoutingProblem) -> Self {
        VehicleRoutingJobInput {
            id: problem.id().to_owned(),
            locations: problem
                .locations()
                .iter()
                .map(|location| JsonLocation::from_problem(location, problem))
                .collect(),
            vehicles: problem
                .vehicles()
                .iter()
                .map(|vehicle| JsonVehicle::from_problem(vehicle, problem))
                .collect(),
            services: problem
                .jobs()
                .iter()
                .filter_map(|job| match job {
                    Job::Service(service) => Some(JsonService::from_problem(service, problem)),
                    _ => None,
                })
                .collect(),
        }
    }
}

pub async fn job_handler(
    Path(path): Path<JobPath>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<VehicleRoutingJobInput>, ApiError> {
    let solver = state
        .solver_manager
        .solver(&path.job_id.to_string())
        .await
        .ok_or(ApiError::NotFound(path.job_id.to_string()))?;

    let problem = solver.problem();

    Ok(Json(VehicleRoutingJobInput::from(problem)))
}

#[derive(Deserialize, JsonSchema)]
pub struct JobNeighborsQuery {
    location_id: usize,
}

pub async fn neighbors_handler(
    Path(path): Path<JobPath>,
    State(state): State<Arc<AppState>>,
    Query(query): Query<JobNeighborsQuery>,
) -> Result<Json<Vec<usize>>, ApiError> {
    let solver = state
        .solver_manager
        .solver(&path.job_id.to_string())
        .await
        .ok_or(ApiError::NotFound(path.job_id.to_string()))?;

    let problem = solver.problem();

    let neighbors = problem
        .neighbors(query.location_id.into())
        .iter()
        .map(|&activity_id| problem.job_activity(activity_id).location_id().get())
        .collect::<Vec<_>>();

    Ok(Json(neighbors))
}
