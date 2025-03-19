use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use geojson::Value::LineString;
use geojson::{Feature, GeoJson, Geometry};
use hermes_core::geopoint::GeoPoint;
use hermes_core::routing::routing_request::RoutingRequest;
use hermes_core::routing_path::RoutingPath;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct RouteResponse {
    path: GeoJson,
}

impl IntoResponse for RouteResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Deserialize)]
pub struct RouteRequestBody {
    start: GeoPoint,
    end: GeoPoint,
}

pub async fn route_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RouteRequestBody>,
) -> Result<RouteResponse, ApiError> {
    let path = state.hermes.route(RoutingRequest {
        start: body.start,
        end: body.end,
        profile: "car",
    });

    path.map(|path| {
        let points: Vec<Vec<f64>> = path
            .legs()
            .iter()
            .flat_map(|leg| leg.points().iter().map(|point| vec![point.lng, point.lat]))
            .collect();

        let feature = Feature {
            bbox: None,
            properties: None,
            foreign_members: None,
            id: None,
            geometry: Some(Geometry::new(LineString(points))),
        };

        RouteResponse {
            path: GeoJson::Feature(feature),
        }
    })
    .map_err(ApiError::InternalServerError)
}
