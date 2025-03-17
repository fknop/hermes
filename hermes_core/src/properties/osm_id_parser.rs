use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property;
use crate::properties::tag_parser::TagParser;

pub struct OsmIdParser;

impl TagParser for OsmIdParser {
    fn handle_way(way: &mut OsmWay) {
        let osm_id = way.osm_id();
        way.properties_mut().insert_usize(Property::OsmId, osm_id);
    }
}
