use rstar::{AABB, Envelope, PointDistance, RTreeObject};

use crate::{
    constants::EARTH_RADIUS_METERS,
    degrees::Degrees,
    distance::{Distance, Meters},
    meters,
};

#[derive(PartialEq, Copy, Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct GeoPoint {
    lat: Degrees,
    lon: Degrees,
}

impl GeoPoint {
    pub fn new(lon: f64, lat: f64) -> GeoPoint {
        GeoPoint {
            lat: Degrees::try_from(lat).unwrap(),
            lon: Degrees::try_from(lon).unwrap(),
        }
    }

    pub fn from_nano(lon: i32, lat: i32) -> GeoPoint {
        GeoPoint {
            lat: Degrees::from(lat),
            lon: Degrees::from(lon),
        }
    }

    pub fn lon_nano(&self) -> i32 {
        self.lon.decimicros()
    }

    pub fn lat_nano(&self) -> i32 {
        self.lat.decimicros()
    }

    pub fn lon(&self) -> f64 {
        self.lon.degrees()
    }

    pub fn lat(&self) -> f64 {
        self.lat.degrees()
    }
}

impl RTreeObject for GeoPoint {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.lon(), self.lat()])
    }
}

impl PointDistance for GeoPoint {
    fn distance_2(&self, point: &<Self::Envelope as Envelope>::Point) -> f64 {
        f64::from(haversine_distance(
            self.lat(),
            self.lon(),
            point[1],
            point[0],
        ))
        .powi(2)
    }
}

impl From<GeoPoint> for geo::Coord<f64> {
    fn from(value: GeoPoint) -> Self {
        geo::Coord {
            x: value.lon(),
            y: value.lat(),
        }
    }
}

impl From<&GeoPoint> for geo::Coord<f64> {
    fn from(value: &GeoPoint) -> Self {
        geo::Coord {
            x: value.lon(),
            y: value.lat(),
        }
    }
}

impl From<&GeoPoint> for geo::Point {
    fn from(value: &GeoPoint) -> Self {
        geo::Point::new(value.lon(), value.lat())
    }
}

impl From<geo::Point> for GeoPoint {
    fn from(value: geo::Point) -> Self {
        GeoPoint::new(value.0.x, value.0.y)
    }
}

impl From<geo::Coord> for GeoPoint {
    fn from(value: geo::Coord) -> Self {
        GeoPoint::new(value.x, value.y)
    }
}

impl GeoPoint {
    pub fn haversine_distance(&self, other: &GeoPoint) -> Distance<Meters> {
        haversine_distance(self.lat(), self.lon(), other.lat(), other.lon())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haversine_distance() {
        let brussels = GeoPoint::new(4.3517, 50.8503);
        let paris = GeoPoint::new(2.3522, 48.8566);

        let distance = brussels.haversine_distance(&paris).value().floor();
        assert_eq!(distance, 263975.0);
    }
}
