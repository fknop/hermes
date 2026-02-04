use std::sync::Arc;

use aide::axum::{
    ApiRouter,
    routing::{get, get_with, post, post_with},
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
        .api_route(
            "/jobs",
            get_with(jobs_handler, |op| op.id("listJobs"))
                .post_with(post_handler, |op| op.id("createJob")),
        )
        .api_route(
            "/jobs/{job_id}",
            get_with(job::job_handler, |op| {
                op.description("Get the job input").id("getJob")
            }),
        )
        .api_route(
            "/jobs/{job_id}/poll",
            get_with(job::poll_handler, |op| {
                op.description("Poll a job that is currently running")
                    .id("pollJob")
            }),
        )
        .api_route(
            "/jobs/{job_id}/start",
            post_with(job::start_handler, |op| {
                op.description("Start a job that was previously created")
                    .id("startJob")
            }),
        )
        .api_route(
            "/jobs/{job_id}/stop",
            post_with(stop_handler, |op| op.id("stopJob")),
        )
        .with_state(state);

    aide::generate::infer_responses(false);

    router
}
