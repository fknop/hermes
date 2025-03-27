use crate::osm::osm_reader::OsmWay;
use crate::properties::car_access_parser::CarAccessParser;
use crate::properties::max_speed_parser::MaxSpeedParser;
use crate::properties::osm_id_parser::OsmIdParser;
use crate::properties::property::Property;

use super::car_average_speed_parser::CarAverageSpeedParser;
use super::property_map::EdgePropertyMap;

pub trait TagParser {
    fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap);
}

pub fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap, property: Property) {
    match property {
        Property::MaxSpeed => MaxSpeedParser::handle_way(way, properties),
        Property::VehicleAccess(vehicle) if vehicle == "car" => {
            CarAccessParser::handle_way(way, properties);
        }
        Property::AverageSpeed(vehicle) if vehicle == "car" => {
            CarAverageSpeedParser::handle_way(way, properties);
        }
        Property::OsmId => OsmIdParser::handle_way(way, properties),
        _ => panic!("Property does not have tag parser"),
    }
}
