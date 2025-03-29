use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use hermes_core::geopoint::GeoPoint;
use hermes_core::graph::Graph;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DebugClosestQuery {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Serialize)]
pub struct DebugClosestResponse {
    pub edge_id: usize,
    pub geojson: GeoJson,
}

impl IntoResponse for DebugClosestResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn debug_closest_handler(
    State(state): State<Arc<AppState>>,
    query: Query<DebugClosestQuery>,
) -> Result<DebugClosestResponse, ApiError> {
    let closest = state
        .hermes
        .closest_edge(String::from("car"), GeoPoint::new(query.lng, query.lat));

    if closest.is_none() {
        return Err(ApiError::BadRequest("Could not find edge".to_string()));
    }

    let edge_id = closest.unwrap();

    let geometry = state.hermes.graph().edge_geometry(edge_id);

    let features: Vec<geojson::Feature> = vec![Feature {
        bbox: None,
        id: None,
        properties: None,
        foreign_members: None,
        geometry: Some(Geometry::new(LineString(
            geometry
                .iter()
                .map(|coordinates| vec![coordinates.lon(), coordinates.lat()])
                .collect(),
        ))),
    }];

    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        foreign_members: None,
        features,
    });

    Ok(DebugClosestResponse { edge_id, geojson })
}
