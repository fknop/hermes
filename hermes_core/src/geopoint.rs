use rstar::{AABB, Envelope, PointDistance, RTreeObject};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

const EARTH_RADIUS: f64 = 6_371_000.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
}

impl RTreeObject for GeoPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.lng, self.lat])
    }
}

impl PointDistance for GeoPoint {
    fn distance_2(&self, point: &<Self::Envelope as Envelope>::Point) -> f64 {
        haversine_distance(self.lat, self.lng, point[1], point[0]).powi(2)
    }
}

impl Into<[f64; 2]> for &GeoPoint {
    fn into(self) -> [f64; 2] {
        let lat_rad = self.lat.to_radians();
        let lon_rad = self.lng.to_radians();
        // Convert to Cartesian
        let x = EARTH_RADIUS * lon_rad;
        let y = EARTH_RADIUS * (lat_rad / 2.0 + PI / 4.0).tan().ln();
        [x, y]
    }
}

impl GeoPoint {
    pub fn haversine_distance(&self, other: &GeoPoint) -> f64 {
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

pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let lat1_rad = lat1.to_radians();
    let lon1_rad = lon1.to_radians();
    let lat2_rad = lat2.to_radians();
    let lon2_rad = lon2.to_radians();

    let delta_lat = lat2_rad - lat1_rad;
    let delta_lon = lon2_rad - lon1_rad;

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    // Calculate distance
    EARTH_RADIUS * c
}
