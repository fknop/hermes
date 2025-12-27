use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use hermes_optimizer::parsers::{parser::DatasetParser, solomon::SolomonParser};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, state::AppState};

#[derive(Serialize)]
pub struct PostBenchmarkResponse {
    job_id: String,
}

#[derive(Deserialize)]
pub struct PostBenchmarkBody {
    category: String,
    name: String,
}

impl IntoResponse for PostBenchmarkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn post_benchmark_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PostBenchmarkBody>,
) -> Result<PostBenchmarkResponse, ApiError> {
    let solver_manager = &state.solver_manager;

    let job_id = Uuid::new_v4().to_string();

    let file = format!("./data/solomon/{}/{}.txt", body.category, body.name);

    let parser = SolomonParser;
    let vrp = parser.parse(&file).ok().unwrap();
    solver_manager.solve(job_id.clone(), vrp).await;
    Ok(PostBenchmarkResponse { job_id })
}
