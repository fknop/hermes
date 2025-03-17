use crate::latlng::LatLng;

pub struct RoutingRequest {
    pub start: LatLng,
    pub end: LatLng,
    pub profile: String,
}
