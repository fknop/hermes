use crate::osm::osm_reader::OsmWay;

use super::{
    max_speed_parser::MaxSpeedParser,
    property::Property,
    property_map::{BACKWARD_EDGE, EdgePropertyMap, FORWARD_EDGE},
    tag_parser::TagParser,
};

pub struct CarAverageSpeedParser;

impl CarAverageSpeedParser {
    fn default_speed_for_highway(highway: &str) -> u8 {
        match highway {
            "motorway" => 120,
            "motorway_link" => 70,

            "trunk" => 70,
            "trunk_link" => 70,

            "primary" => 60,
            "primary_link" => 60,

            "secondary" => 50,
            "secondary_link" => 40,

            "tertiary" => 30,
            "tertiary_link" => 30,

            "unclassified" => 30,
            "residential" => 30,
            "living_street" => 5,
            "service" => 20,

            "road" => 20,
            "track" => 15,

            _ => 30,
        }
    }

    fn parse_average_speed(way: &OsmWay) -> u8 {
        let max_speed = MaxSpeedParser::parse_max_speed(way);

        match max_speed {
            Some(max_speed) => max_speed,
            None => {
                CarAverageSpeedParser::default_speed_for_highway(way.tag("highway").unwrap_or(""))
            }
        }
    }
}

impl TagParser for CarAverageSpeedParser {
    fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        let car_average_speed = CarAverageSpeedParser::parse_average_speed(way);
        properties.insert_u8(
            Property::AverageSpeed(String::from("car")),
            FORWARD_EDGE,
            car_average_speed,
        );
        properties.insert_u8(
            Property::AverageSpeed(String::from("car")),
            BACKWARD_EDGE,
            car_average_speed,
        );
    }
}
