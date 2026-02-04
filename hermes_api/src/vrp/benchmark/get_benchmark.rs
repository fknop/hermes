use axum::{Json, extract::Path};
use hermes_optimizer::parsers::{parser::DatasetParser, solomon::SolomonParser};

use crate::{error::ApiError, vrp::job::VehicleRoutingJobInput};

pub async fn get_benchmark_handler(
    Path((category, name)): Path<(String, String)>,
) -> Result<Json<VehicleRoutingJobInput>, ApiError> {
    let file = format!("./data/solomon/{category}/{name}.txt");

    let parser = SolomonParser;
    if let Ok(vrp) = parser.parse(&file) {
        Ok(Json(VehicleRoutingJobInput::from(&vrp)))
    } else {
        Err(ApiError::BadRequest(String::from("Invalid input")))
    }
}
