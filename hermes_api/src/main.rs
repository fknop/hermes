mod debug;
mod error;
mod route;
mod state;

use crate::debug::closest::debug_closest_handler;
use crate::route::route::route_handler;
use crate::state::AppState;
use axum::http::Method;
use axum::routing::{get, post};
use axum::{Router, serve};
use hermes_core::hermes::Hermes;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let hermes = Hermes::new_from_osm("./data/osm/brussels_capital_region-latest.osm.pbf");

    let app_state = Arc::new(AppState { hermes });

    let cors_layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/route", post(route_handler))
        .route("/debug/closest", get(debug_closest_handler))
        .layer(ServiceBuilder::new().layer(cors_layer))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    serve(listener, app).await.unwrap();
}
