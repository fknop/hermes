use serde::Deserialize;

use crate::define_index_newtype;

define_index_newtype!(LocationIdx, Location);

pub struct Location {
    /// id is the index of the location in the locations list
    id: LocationIdx,
    x: f64,
    y: f64,
}

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

impl Location {
    pub fn from_cartesian<Id: Into<LocationIdx>>(id: Id, x: f64, y: f64) -> Self {
        Self {
            id: id.into(),
            x,
            y,
        }
    }

    pub fn from_lat_lon<Id: Into<LocationIdx>>(id: Id, lat: f64, lon: f64) -> Self {
        Self {
            id: id.into(),
            x: lon,
            y: lat,
        }
    }

    pub fn id(&self) -> LocationIdx {
        self.id
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn lon(&self) -> f64 {
        self.x
    }

    pub fn lat(&self) -> f64 {
        self.y
    }

    pub fn euclidian_distance(&self, to: &Location) -> f64 {
        let delta_x = self.x - to.x;
        let delta_y = self.y - to.y;
        (delta_x * delta_x + delta_y * delta_y).sqrt()
    }

    pub fn haversine_distance(&self, to: &Location) -> f64 {
        let lat1_rad = self.lat().to_radians();
        let lon1_rad = self.lon().to_radians();
        let lat2_rad = to.lat().to_radians();
        let lon2_rad = to.lon().to_radians();

        let delta_lat = lat2_rad - lat1_rad;
        let delta_lon = lon2_rad - lon1_rad;

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        // Calculate distance
        EARTH_RADIUS_METERS * c
    }

    pub fn bearing(&self, dest: &Self) -> f64 {
        let lat1_rad = self.lat().to_radians();
        let lon1_rad = self.lon().to_radians();
        let lat2_rad = dest.lat().to_radians();
        let lon2_rad = dest.lon().to_radians();

        let delta_lon = lon2_rad - lon1_rad;

        let y = delta_lon.sin() * lat2_rad.cos();
        let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * delta_lon.cos();

        let bearing_rad = y.atan2(x);
        let bearing_deg = bearing_rad.to_degrees();

        (bearing_deg + 360.0) % 360.0
    }
}

impl From<&Location> for geo::Point<f64> {
    fn from(location: &Location) -> Self {
        geo::Point::new(location.x(), location.y())
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
