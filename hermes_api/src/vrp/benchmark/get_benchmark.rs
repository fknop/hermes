use axum::{Json, extract::Path};
use hermes_optimizer::parsers::parser::parse_dataset;

use crate::{error::ApiError, vrp::job::VehicleRoutingJobInput};

pub async fn get_benchmark_handler(
    Path((category, name)): Path<(String, String)>,
) -> Result<Json<VehicleRoutingJobInput>, ApiError> {
    let file = format!("./data/vrptw/solomon/{category}/{name}.txt");

    if let Ok(vrp) = parse_dataset(&file) {
        Ok(Json(VehicleRoutingJobInput::from(&vrp)))
    } else {
        Err(ApiError::BadRequest(String::from("Invalid input")))
    }
}
