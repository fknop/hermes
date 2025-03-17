mod error;
mod route;
mod state;

use crate::route::route::route_handler;
use crate::state::AppState;
use axum::routing::{get, post};
use axum::{Router, serve};
use hermes_core::hermes::Hermes;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let hermes = Hermes::new_from_osm("./data/osm/brussels_capital_region-latest.osm.pbf");

    let app_state = Arc::new(AppState { hermes });

    let app = Router::new()
        .route("/route", post(route_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    serve(listener, app).await.unwrap();
}
