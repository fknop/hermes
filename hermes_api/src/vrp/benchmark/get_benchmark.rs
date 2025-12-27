use axum::{extract::Path, response::IntoResponse};
use hermes_optimizer::{
    parsers::{parser::DatasetParser, solomon::SolomonParser},
    problem::{service::Service, vehicle::Vehicle},
};
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

#[derive(Deserialize)]
pub struct GetBenchmarkQuery {
    category: String,
    name: String,
}

#[derive(Serialize)]
pub struct GetBenchmarkService {}

#[derive(Serialize)]
pub struct BenchmarkLocation {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
pub struct GetBenchmarkResponse {
    locations: Vec<BenchmarkLocation>,
    services: Vec<Service>,
    vehicles: Vec<Vehicle>,
}

impl IntoResponse for GetBenchmarkResponse {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::OK, axum::Json(self)).into_response()
    }
}

pub async fn get_benchmark_handler(
    Path((category, name)): Path<(String, String)>,
) -> Result<GetBenchmarkResponse, ApiError> {
    let file = format!("./data/solomon/{category}/{name}.txt");

    let parser = SolomonParser;
    if let Ok(vrp) = parser.parse(&file) {
        Ok(GetBenchmarkResponse {
            locations: vrp
                .locations()
                .iter()
                .map(|loc| BenchmarkLocation {
                    x: loc.x(),
                    y: loc.y(),
                })
                .collect(),
            services: vrp.services_iter().cloned().collect::<Vec<_>>(),
            vehicles: vrp.vehicles().to_vec(),
        })
    } else {
        Err(ApiError::BadRequest(String::from("Invalid input")))
    }
}
