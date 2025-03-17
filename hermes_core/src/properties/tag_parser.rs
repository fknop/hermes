use crate::osm::osm_reader::OsmWay;
use crate::properties::car_access_parser::CarAccessParser;
use crate::properties::max_speed_parser::MaxSpeedParser;
use crate::properties::osm_id_parser::OsmIdParser;
use crate::properties::property::Property;

pub trait TagParser {
    fn handle_way(way: &mut OsmWay);
}

pub fn handle_way(way: &mut OsmWay, property: Property) {
    match property {
        Property::MaxSpeed => MaxSpeedParser::handle_way(way),
        Property::VehicleAccess(vehicle) if vehicle == "car" => {
            CarAccessParser::handle_way(way);
        }
        Property::OsmId => OsmIdParser::handle_way(way),
        _ => panic!("Property does not have tag parser"),
    }
}
