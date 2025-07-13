use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::problem::{
    location::Location, service::Service, travel_cost_matrix::TravelCostMatrix, vehicle::Vehicle,
    vehicle_routing_problem::VehicleRoutingProblemBuilder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Deserialize)]
pub struct PostRequestBody {
    locations: Vec<Location>,
    vehicles: Vec<Vehicle>,
    services: Vec<Service>,
    travel_costs: TravelCostMatrix,
}

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
    Json(body): Json<PostRequestBody>,
) -> Result<PostResponse, ApiError> {
    let solver_manager = &state.solver_manager;

    let job_id = Uuid::new_v4().to_string();

    let mut builder = VehicleRoutingProblemBuilder::default();

    builder
        .set_services(body.services)
        .set_vehicles(body.vehicles)
        .set_locations(body.locations)
        .set_travel_costs(body.travel_costs);

    let vrp = builder.build();

    solver_manager.solve(job_id.clone(), vrp).await;

    Ok(PostResponse { job_id })
}
