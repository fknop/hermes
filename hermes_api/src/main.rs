mod docs;
mod error;
mod landmarks;
mod pagination;
mod route;
mod state;
mod vrp;

use crate::docs::docs_routes;
use crate::get_landmarks::get_landmarks;
use crate::route::route_handler::route_handler;
use crate::state::AppState;
use crate::vrp::routes::vrp_routes;
use aide::openapi::OpenApi;
use aide::transform::TransformOpenApi;
use axum::http::Method;
use axum::routing::{any, get, post};
use axum::{Extension, Router, serve};
use hermes_matrix_providers::travel_matrix_client::TravelMatrixClient;
use hermes_optimizer::solver::solver_manager::SolverManager;
use hermes_routing::hermes::Hermes;
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
    dotenvy::from_filename("./.env.local").ok();
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    aide::generate::extract_schemas(true);

    let hermes = Hermes::from_directory("./data/be");

    let state = Arc::new(AppState {
        hermes,
        solver_manager: SolverManager::default(),
        matrix_client: TravelMatrixClient::default(),
    });

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let mut api = OpenApi::default();

    let app = aide::axum::ApiRouter::new()
        .nest_api_service("/docs", docs_routes(state.clone()))
        .route("/route", post(route_handler))
        .route("/landmarks", get(get_landmarks))
        .nest_api_service("/vrp", vrp_routes(state.clone()))
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
        .finish_api_with(&mut api, api_docs)
        .layer(ServiceBuilder::new().layer(cors_layer))
        .layer(Extension(Arc::new(api)))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    serve(listener, app).await.unwrap();
}

fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Hermes Open API")
}
