use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property;
use crate::properties::property_map::{BACKWARD_EDGE, EdgePropertyMap, FORWARD_EDGE};
use crate::properties::tag_parser::TagParser;

pub struct MaxSpeedParser;

fn get_max_speed(way: &OsmWay) -> Option<u8> {
    match way.get_tag("maxspeed") {
        Some("walk") => Some(5),
        Some("none") => Some(150),
        Some(max_speed) => max_speed.parse::<u8>().ok(),
        None => None,
    }
}

// https://wiki.openstreetmap.org/wiki/Key:maxspeed
impl TagParser for MaxSpeedParser {
    fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        if let Some(max_speed) = get_max_speed(way) {
            properties.insert_u8(Property::MaxSpeed, FORWARD_EDGE, max_speed);
            properties.insert_u8(Property::MaxSpeed, BACKWARD_EDGE, max_speed);
        }
    }
}
