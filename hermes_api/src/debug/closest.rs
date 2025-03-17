use crate::state::AppState;
use axum::extract::{Query, State};
use hermes_core::latlng::LatLng;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct DebugClosestQuery {
    pub lat: f64,
    pub lng: f64,
}

pub async fn debug_closest_handler(
    State(state): State<Arc<AppState>>,
    query: Query<DebugClosestQuery>,
) -> Result<String, String> {
    let closest = state.hermes.closest_edge(LatLng {
        lat: query.lat,
        lng: query.lng,
    });

    closest
        .map(|value| value.to_string())
        .ok_or(String::from("Closest edge query not found"))
}
