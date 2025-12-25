use crate::osm::osm_reader::OsmWay;
use crate::properties::car_access_parser::CarAccessParser;
use crate::properties::max_speed_parser::MaxSpeedParser;
use crate::properties::osm_id_parser::OsmIdParser;
use crate::properties::property::Property;

use super::car_average_speed_parser::CarAverageSpeedParser;
use super::property_map::EdgePropertyMap;

pub trait TagParser {
    fn parse_way(way: &OsmWay, properties: &mut EdgePropertyMap);
}

pub fn parse_way_tags(way: &OsmWay, properties: &mut EdgePropertyMap, property: Property) {
    match property {
        Property::MaxSpeed => MaxSpeedParser::parse_way(way, properties),
        Property::CarVehicleAccess => {
            CarAccessParser::parse_way(way, properties);
        }
        Property::CarAverageSpeed => {
            CarAverageSpeedParser::parse_way(way, properties);
        }
        Property::OsmId => OsmIdParser::parse_way(way, properties),
    }
}
