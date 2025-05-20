use crate::error::ApiError;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use geojson::Value::Point;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct GetLandmarksResponse(GeoJson);

impl IntoResponse for GetLandmarksResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

pub async fn get_landmarks(
    State(state): State<Arc<AppState>>,
) -> Result<GetLandmarksResponse, ApiError> {
    let landmarks = state.hermes.get_landmarks();

    /*
    let forward_feature = Feature {
        geometry: Some(Geometry::new(MultiPoint(points))),
        properties: Some(properties),
        ..Default::default()
    }; */

    let features = landmarks
        .iter()
        .map(|&coordinates| Feature {
            geometry: Some(Geometry::new(Point(vec![
                coordinates.lon(),
                coordinates.lat(),
            ]))),
            properties: None,
            id: None,
            bbox: None,
            foreign_members: None,
        })
        .collect();

    Ok(GetLandmarksResponse(GeoJson::FeatureCollection(
        FeatureCollection {
            features,
            bbox: None,
            foreign_members: None,
        },
    )))
}
