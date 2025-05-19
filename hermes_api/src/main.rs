mod error;
mod route;
mod state;

use crate::route::route_handler::route_handler;
use crate::state::AppState;
use axum::http::Method;
use axum::routing::post;
use axum::{Router, serve};
use hermes_core::hermes::Hermes;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let hermes = Hermes::from_directory("./data");

    let app_state = Arc::new(AppState { hermes });

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/route", post(route_handler))
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    serve(listener, app).await.unwrap();
}
