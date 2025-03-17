use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use hermes_core::latlng::LatLng;
use hermes_core::routing::routing_request::RoutingRequest;
use hermes_core::routing_path::RoutingPath;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct RouteResponse {
    path: RoutingPath,
}

impl IntoResponse for RouteResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Deserialize)]
pub struct RouteRequestBody {
    start: LatLng,
    end: LatLng,
}

pub async fn route_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RouteRequestBody>,
) -> Result<RouteResponse, ApiError> {
    let path = state.hermes.route(RoutingRequest {
        start: body.start,
        end: body.end,
        profile: "car".to_string(),
    });

    path.map(|path| RouteResponse { path })
        .map_err(ApiError::InternalServerError)
}
