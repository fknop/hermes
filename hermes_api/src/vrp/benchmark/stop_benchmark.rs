use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};

use crate::{error::ApiError, state::AppState};

pub async fn stop_benchmark_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<(), ApiError> {
    let solver_manager = &state.solver_manager;

    solver_manager.stop(&job_id).await;

    Ok(())
}
