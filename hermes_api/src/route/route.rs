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
pub struct GeoPointBody {
    lat: f64,
    lon: f64,
}

impl From<GeoPointBody> for GeoPoint {
    fn from(value: GeoPointBody) -> Self {
        GeoPoint::new(value.lat, value.lon)
    }
}

#[derive(Deserialize)]
pub struct RouteRequestBody {
    start: GeoPointBody,
    end: GeoPointBody,
}

pub async fn route_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RouteRequestBody>,
) -> Result<RouteResponse, ApiError> {
    let path = state.hermes.route(RoutingRequest {
        start: body.start.into(),
        end: body.end.into(),
        profile: String::from("car"),
    });

    path.map(|path| {
        let points: Vec<Vec<f64>> = path
            .legs()
            .iter()
            .flat_map(|leg| {
                leg.points()
                    .iter()
                    .map(|point| vec![point.lon(), point.lat()])
            })
            .collect();

        println!("found points with {:?}", points.len());

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
