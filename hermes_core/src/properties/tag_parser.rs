use crate::osm::osm_reader::OsmWay;
use crate::properties::car_access_parser::CarAccessParser;
use crate::properties::max_speed_parser::MaxSpeedParser;
use crate::properties::property::Property;
use crate::properties::property_map::EdgePropertyMap;

pub trait TagParser {
    fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap);
}

fn handle_way(way: &OsmWay, property: Property, properties: &mut EdgePropertyMap) {
    match property {
        Property::MaxSpeed => MaxSpeedParser::handle_way(way, properties),
        Property::VehicleAccess(vehicle) if vehicle == "car" => {
            CarAccessParser::handle_way(way, properties)
        }
        _ => panic!("Property does not have tag parser"),
    }
}
