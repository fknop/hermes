mod error;
mod landmarks;
mod route;
mod state;
mod vrp;

use crate::get_landmarks::get_landmarks;
use crate::route::route_handler::route_handler;
use crate::state::AppState;
use axum::http::Method;
use axum::routing::{any, get, post};
use axum::{Router, serve};
use hermes_optimizer_core::solver::solver_manager::SolverManager;
use hermes_routing_core::hermes::Hermes;
use landmarks::get_landmarks;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let hermes = Hermes::from_directory("./data");

    let app_state = Arc::new(AppState {
        hermes,
        solver_manager: SolverManager::default(),
    });

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/route", post(route_handler))
        .route("/landmarks", get(get_landmarks))
        .route("/vrp/ws", any(vrp::ws::handler))
        .route("/vrp", post(vrp::post::post_handler))
        .route("/vrp/poll/{job_id}", get(vrp::poll::poll_handler))
        .route(
            "/vrp/benchmark",
            post(vrp::benchmark::post_benchmark::post_benchmark_handler),
        )
        .route(
            "/vrp/benchmark/{category}/{name}",
            get(vrp::benchmark::get_benchmark::get_benchmark_handler),
        )
        .route(
            "/vrp/benchmark/poll/{job_id}",
            get(vrp::benchmark::poll_benchmark::poll_handler),
        )
        .route(
            "/vrp/benchmark/stop/{job_id}",
            post(vrp::benchmark::stop_benchmark::stop_benchmark_handler),
        )
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    serve(listener, app).await.unwrap();
}
