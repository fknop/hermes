use crate::latlng::LatLng;
use serde::{Deserialize, Serialize};
#[derive(Serialize)]

pub struct RoutingPathItem {
    distance: f64,
    time: usize,
    points: Vec<LatLng>,
}

impl RoutingPathItem {
    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn time(&self) -> usize {
        self.time
    }

    pub fn points(&self) -> &[LatLng] {
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

#[derive(Serialize)]

pub struct RoutingPath {
    items: Vec<RoutingPathItem>,
}

impl RoutingPath {
    pub fn new(items: Vec<RoutingPathItem>) -> RoutingPath {
        RoutingPath { items }
    }

    pub fn legs(&self) -> &[RoutingPathItem] {
        &self.items
    }
}
