use crate::latlng::LatLng;

struct RoutingPathItem {
    distance: f64,
    time: f64,
    points: Vec<LatLng>,
}

pub struct RoutingPath {
    items: Vec<RoutingPathItem>,
}

impl RoutingPath {
    pub fn new() -> RoutingPath {
        RoutingPath { items: Vec::new() }
    }
}
