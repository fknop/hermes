use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property;
use crate::properties::tag_parser::TagParser;

use super::property_map::EdgePropertyMap;

pub struct OsmIdParser;

impl TagParser for OsmIdParser {
    fn parse_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        let osm_id = way.osm_id();
        properties.insert_usize(Property::OsmId, osm_id);
    }
}
