use std::sync::Arc;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};

use crate::{
    state::AppState,
    vrp::{
        job::{self, stop_handler},
        jobs::jobs_handler,
        post_handler::post_handler,
    },
};

pub fn vrp_routes(state: Arc<AppState>) -> ApiRouter {
    aide::generate::infer_responses(true);
    let router = ApiRouter::new()
        .api_route("/jobs", get(jobs_handler).post(post_handler))
        .api_route("/jobs/{job_id}/poll", get(job::poll_handler))
        .api_route("/jobs/{job_id}/start", post(job::start_handler))
        .api_route("/jobs/{job_id}/stop", post(stop_handler))
        .with_state(state);

    aide::generate::infer_responses(false);

    router
}
