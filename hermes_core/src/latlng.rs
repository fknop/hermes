use geo_types::Point;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

const EARTH_RADIUS: f64 = 6_371_000.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl Into<Point> for LatLng {
    fn into(self) -> Point {
        let lat_rad = self.lat.to_radians();
        let lon_rad = self.lng.to_radians();
        // Convert to Cartesian
        let x = EARTH_RADIUS * lat_rad.cos() * lon_rad.cos();
        let y = EARTH_RADIUS * lat_rad.cos() * lon_rad.sin();
        Point::new(x, y)
    }
}

impl Into<[f64; 2]> for &LatLng {
    fn into(self) -> [f64; 2] {
        let lat_rad = self.lat.to_radians();
        let lon_rad = self.lng.to_radians();
        // Convert to Cartesian
        let x = EARTH_RADIUS * lon_rad;
        let y = EARTH_RADIUS * (lat_rad / 2.0 + PI / 4.0).tan().ln();
        [x, y]
    }
}

impl LatLng {
    pub fn haversine_distance(&self, other: &LatLng) -> f64 {
        let lat1 = self.lat.to_radians();
        let lng1 = self.lng.to_radians();
        let lat2 = other.lat.to_radians();
        let lng2 = other.lng.to_radians();

        let dlat = lat2 - lat1;
        let dlng = lng2 - lng1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlng / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        // Calculate distance
        EARTH_RADIUS * c
    }
}
