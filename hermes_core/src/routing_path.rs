
use crate::{
    distance::{Distance, Meters},
    geopoint::GeoPoint,
};
pub struct RoutingPathItem {
    distance: Distance<Meters>,
    time: usize,
    points: Vec<GeoPoint>,
}

impl RoutingPathItem {
    pub fn distance(&self) -> Distance<Meters> {
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
    pub fn new(distance: Distance<Meters>, time: usize, points: Vec<GeoPoint>) -> RoutingPathItem {
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

    pub fn legs(&self) -> &[RoutingPathItem] {
        &self.items
    }
}
