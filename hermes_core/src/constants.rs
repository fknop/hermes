use crate::weighting::Weight;

pub(crate) const INVALID_NODE: usize = usize::MAX;
pub(crate) const INVALID_EDGE: usize = usize::MAX;
pub(crate) const MAX_WEIGHT: Weight = u32::MAX;
pub(crate) const MAX_DURATION: Weight = u32::MAX;

pub(crate) const EARTH_RADIUS_METERS: f64 = 6_371_000.0;

pub(crate) const DISTANCE_INFLUENCE: f64 = 50.0;

pub(crate) const MPH_TO_KPH: f32 = 1.60934;
