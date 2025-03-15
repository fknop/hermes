use rstar::{AABB, RTreeObject};

#[derive(Copy, Clone)]
pub struct LatLng {
    pub lat: f64,
    pub lng: f64,
}

impl Into<[f64; 2]> for &LatLng {
    fn into(self) -> [f64; 2] {
        [self.lng, self.lat]
    }
}
