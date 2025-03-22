use crate::geopoint::GeoPoint;

pub struct RoutingRequest {
    pub start: GeoPoint,
    pub end: GeoPoint,
    pub profile: &'static str,
}
