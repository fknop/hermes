use serde::{Deserialize, Serialize};

const EARTH_RADIUS: f64 = 6_371_000.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl Into<[f64; 2]> for &LatLng {
    fn into(self) -> [f64; 2] {
        [self.lng, self.lat]
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
