use crate::{
    distance::{Distance, Meters},
    geopoint::GeoPoint,
    weighting::Milliseconds,
};

pub struct RoutingPathLeg {
    distance: Distance<Meters>,
    time: Milliseconds,
    points: Vec<GeoPoint>,
}

impl RoutingPathLeg {
    pub fn distance(&self) -> Distance<Meters> {
        self.distance
    }

    pub fn time(&self) -> Milliseconds {
        self.time
    }

    pub fn points(&self) -> &[GeoPoint] {
        &self.points
    }
}

impl RoutingPathLeg {
    pub fn new(
        distance: Distance<Meters>,
        time: Milliseconds,
        points: Vec<GeoPoint>,
    ) -> RoutingPathLeg {
        RoutingPathLeg {
            points,
            distance,
            time,
        }
    }
}

pub struct RoutingPath {
    legs: Vec<RoutingPathLeg>,
    distance: Distance<Meters>,
    time: Milliseconds,
}

impl RoutingPath {
    pub fn new(legs: Vec<RoutingPathLeg>) -> RoutingPath {
        let distance = legs.iter().map(|leg| leg.distance()).sum();
        let time = legs.iter().map(|leg| leg.time()).sum();
        RoutingPath {
            legs,
            distance,
            time,
        }
    }

    pub fn distance(&self) -> Distance<Meters> {
        self.distance
    }

    pub fn time(&self) -> Milliseconds {
        self.time
    }

    pub fn legs(&self) -> &[RoutingPathLeg] {
        &self.legs
    }
}
