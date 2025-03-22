use serde::Serialize;

use crate::geopoint::GeoPoint;
#[derive(Serialize)]
pub struct RoutingPathItem {
    distance: f64,
    time: usize,
    points: Vec<GeoPoint>,
}

impl RoutingPathItem {
    pub fn distance(&self) -> f64 {
        self.distance
    }

    pub fn time(&self) -> usize {
        self.time
    }

    pub fn points(&self) -> &[GeoPoint] {
        &self.points
    }
}

impl RoutingPathItem {
    pub fn new(distance: f64, time: usize, points: Vec<GeoPoint>) -> RoutingPathItem {
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
