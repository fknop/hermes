use crate::{
    distance::{Distance, Meters},
    geopoint::GeoPoint,
};

pub struct RoutingPathLeg {
    distance: Distance<Meters>,
    time: usize,
    points: Vec<GeoPoint>,
}

impl RoutingPathLeg {
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

impl RoutingPathLeg {
    pub fn new(distance: Distance<Meters>, time: usize, points: Vec<GeoPoint>) -> RoutingPathLeg {
        RoutingPathLeg {
            points,
            distance,
            time,
        }
    }
}

pub struct RoutingPath {
    legs: Vec<RoutingPathLeg>,
}

impl RoutingPath {
    pub fn new(legs: Vec<RoutingPathLeg>) -> RoutingPath {
        RoutingPath { legs }
    }

    pub fn legs(&self) -> &[RoutingPathLeg] {
        &self.legs
    }
}
