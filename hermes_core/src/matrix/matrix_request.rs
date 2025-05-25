use crate::geopoint::GeoPoint;

pub struct MatrixRequestOptions {
    pub include_debug_info: Option<bool>,
}

pub struct MatrixRequest {
    pub sources: Vec<GeoPoint>,
    pub targets: Vec<GeoPoint>,
    pub profile: String,
    pub options: Option<MatrixRequestOptions>,
}
