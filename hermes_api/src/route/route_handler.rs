use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use geojson::Value::{LineString, MultiPoint};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonValue};
use hermes_core::geopoint::GeoPoint;
use hermes_core::routing::routing_request::{
    RoutingAlgorithm, RoutingRequest, RoutingRequestOptions,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct RouteResponse(GeoJson);

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
        GeoPoint::new(value.lon, value.lat)
    }
}

#[derive(Deserialize)]
pub struct RouteRequestBody {
    start: GeoPointBody,
    end: GeoPointBody,
    include_debug_info: Option<bool>,
    algorithm: Option<RoutingAlgorithm>,
}

pub async fn route_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RouteRequestBody>,
) -> Result<RouteResponse, ApiError> {
    let result = state.hermes.route(RoutingRequest {
        start: body.start.into(),
        end: body.end.into(),
        profile: String::from("car"),
        options: Some(RoutingRequestOptions {
            algorithm: body.algorithm,
            include_debug_info: body.include_debug_info,
        }),
    });

    result
        .map(|result| {
            let mut features: Vec<Feature> = vec![];

            let points: Vec<Vec<f64>> = result
                .path
                .legs()
                .iter()
                .flat_map(|leg| {
                    leg.points()
                        .iter()
                        .map(|point| vec![point.lon(), point.lat()])
                })
                .collect();

            let mut properties = serde_json::Map::new();
            properties.insert(
                String::from("id"),
                JsonValue::String(String::from("polyline")),
            );

            let feature = Feature {
                bbox: None,
                properties: Some(properties),
                foreign_members: None,
                id: None,
                geometry: Some(Geometry::new(LineString(points))),
            };

            features.push(feature);

            if let Some(debug) = result.debug {
                if !debug.forward_visited_nodes.is_empty() {
                    let points = debug
                        .forward_visited_nodes
                        .iter()
                        .map(|point| vec![point.lon(), point.lat()])
                        .collect();

                    let mut properties = serde_json::Map::new();
                    properties.insert(
                        String::from("id"),
                        JsonValue::String(String::from("forward_visited_nodes")),
                    );

                    let forward_feature = Feature {
                        geometry: Some(Geometry::new(MultiPoint(points))),
                        properties: Some(properties),
                        ..Default::default()
                    };

                    features.push(forward_feature);
                }

                if !debug.backward_visited_nodes.is_empty() {
                    let points = debug
                        .backward_visited_nodes
                        .iter()
                        .map(|point| vec![point.lon(), point.lat()])
                        .collect();

                    let mut properties = serde_json::Map::new();
                    properties.insert(
                        String::from("id"),
                        JsonValue::String(String::from("backward_visited_nodes")),
                    );

                    let backward_feature = Feature {
                        properties: Some(properties),
                        geometry: Some(Geometry::new(MultiPoint(points))),
                        ..Default::default()
                    };

                    features.push(backward_feature);
                }
            }

            RouteResponse(GeoJson::FeatureCollection(FeatureCollection {
                bbox: None,
                features,
                foreign_members: None,
            }))
        })
        .map_err(ApiError::InternalServerError)
}
