use crate::properties::car_access_parser::CarAccessParser;
use crate::properties::max_speed_parser::MaxSpeedParser;
use crate::properties::tag_parser::TagParser;

#[derive(Eq, Hash, PartialEq)]
pub enum Property {
    MaxSpeed,
    VehicleAccess(String),
}
