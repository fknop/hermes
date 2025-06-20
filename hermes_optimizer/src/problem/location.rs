pub type LocationId = usize;
pub struct Location {
    /// id is the index of the location in the locations list
    id: LocationId,
    lon: f64,
    lat: f64,
}

const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

impl Location {
    pub fn id(&self) -> LocationId {
        self.id
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn haversine_distance(&self, to: &Location) -> f64 {
        let lat1_rad = self.lat.to_radians();
        let lon1_rad = self.lon.to_radians();
        let lat2_rad = to.lat.to_radians();
        let lon2_rad = to.lon.to_radians();

        let delta_lat = lat2_rad - lat1_rad;
        let delta_lon = lon2_rad - lon1_rad;

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        // Calculate distance
        EARTH_RADIUS_METERS * c
    }
}
