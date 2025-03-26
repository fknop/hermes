use rstar::{AABB, Envelope, PointDistance, RTreeObject};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

use crate::{
    constants::EARTH_RADIUS_METERS,
    distance::{Distance, Meters, meters},
};

#[derive(
    PartialEq,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
pub struct GeoPoint {
    pub lat: f64,
    pub lon: f64,
}

impl RTreeObject for GeoPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.lon, self.lat])
    }
}

impl PointDistance for GeoPoint {
    fn distance_2(&self, point: &<Self::Envelope as Envelope>::Point) -> f64 {
        f64::from(haversine_distance(self.lat, self.lon, point[1], point[0])).powi(2)
    }
}

impl Into<[f64; 2]> for &GeoPoint {
    fn into(self) -> [f64; 2] {
        let lat_rad = self.lat.to_radians();
        let lon_rad = self.lon.to_radians();
        // Convert to Cartesian
        let x = EARTH_RADIUS_METERS * lon_rad;
        let y = EARTH_RADIUS_METERS * (lat_rad / 2.0 + PI / 4.0).tan().ln();
        [x, y]
    }
}

impl GeoPoint {
    pub fn distance(&self, other: &GeoPoint) -> Distance<Meters> {
        haversine_distance(self.lat, self.lon, other.lat, other.lon)
    }
}

pub fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> Distance<Meters> {
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
    meters!(EARTH_RADIUS_METERS * c)
}
