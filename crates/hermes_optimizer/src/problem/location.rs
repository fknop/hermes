use geo::{Bearing, Distance, Euclidean, Haversine};

use crate::define_index_newtype;

define_index_newtype!(LocationIdx, Location);

pub struct Location {
    point: geo::Point,
}

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

impl Location {
    pub fn from_cartesian(x: f64, y: f64) -> Self {
        Self {
            point: geo::Point::new(x, y),
        }
    }

    pub fn from_lat_lon(lat: f64, lon: f64) -> Self {
        Self {
            point: geo::Point::new(lon, lat),
        }
    }

    pub fn x(&self) -> f64 {
        self.point.x()
    }

    pub fn y(&self) -> f64 {
        self.point.y()
    }

    pub fn lon(&self) -> f64 {
        self.point.x()
    }

    pub fn lat(&self) -> f64 {
        self.point.y()
    }

    pub fn euclidean_distance(&self, to: &Location) -> f64 {
        let euclidean = Euclidean;
        euclidean.distance(&self.point, &to.point)
    }

    pub fn haversine_distance(&self, to: &Location) -> f64 {
        let haversine = Haversine;

        haversine.distance(self.point, to.point)
    }

    pub fn bearing(&self, dest: &Self) -> f64 {
        let haversine = Haversine;
        haversine.bearing(self.point, dest.point)
    }
}

impl From<&Location> for geo::Point<f64> {
    fn from(location: &Location) -> Self {
        location.point
    }
}

impl From<&Location> for geo::Coord<f64> {
    fn from(val: &Location) -> Self {
        geo::Coord {
            x: val.x(),
            y: val.y(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn euclidean_distance() {
        let loc1 = super::Location::from_cartesian(0.0, 0.0);
        let loc2 = super::Location::from_cartesian(3.0, 4.0);

        let dist = loc1.euclidean_distance(&loc2);
        assert_eq!(dist, 5.0);
    }
}
