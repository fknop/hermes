use crate::latlng::LatLng;

pub struct RoutingPathItem {
    distance: f64,
    time: usize,
    points: Vec<LatLng>,
}

impl RoutingPathItem {
    pub fn get_distance(&self) -> f64 {
        self.distance
    }

    pub fn get_time(&self) -> usize {
        self.time
    }

    pub fn get_points(&self) -> &[LatLng] {
        &self.points
    }
}

impl RoutingPathItem {
    pub fn new(distance: f64, time: usize, points: Vec<LatLng>) -> RoutingPathItem {
        RoutingPathItem {
            points,
            distance,
            time,
        }
    }
}

pub struct RoutingPath {
    items: Vec<RoutingPathItem>,
}

impl RoutingPath {
    pub fn new(items: Vec<RoutingPathItem>) -> RoutingPath {
        RoutingPath { items }
    }

    pub fn get_legs(&self) -> &[RoutingPathItem] {
        &self.items
    }
}
