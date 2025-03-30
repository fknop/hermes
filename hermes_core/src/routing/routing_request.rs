use serde::Deserialize;

use crate::geopoint::GeoPoint;

#[derive(Clone, Copy, Deserialize)]
pub enum RoutingAlgorithm {
    Dijkstra,
    Astar,
    BidirectionalAstar,
}

pub struct RoutingRequestOptions {
    pub include_debug_info: Option<bool>,
    pub algorithm: Option<RoutingAlgorithm>,
}

pub struct RoutingRequest {
    pub start: GeoPoint,
    pub end: GeoPoint,
    pub profile: String,
    pub options: Option<RoutingRequestOptions>,
}
