use crate::{edge_direction::EdgeDirection, osm::osm_reader::OsmWay};

use super::{
    max_speed_parser::MaxSpeedParser, property::Property, property_map::EdgePropertyMap,
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

    fn parse_average_speed(way: &OsmWay) -> f32 {
        let max_speed = MaxSpeedParser::parse_max_speed(way);

        match max_speed {
            Some(max_speed) => max_speed,
            None => {
                CarAverageSpeedParser::default_speed_for_highway(way.tag("highway").unwrap_or(""))
                    as f32
            }
        }
    }
}

impl TagParser for CarAverageSpeedParser {
    fn parse_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        let car_average_speed = CarAverageSpeedParser::parse_average_speed(way);
        properties.insert_f32(
            Property::CarAverageSpeed,
            EdgeDirection::Forward,
            car_average_speed,
        );
        properties.insert_f32(
            Property::CarAverageSpeed,
            EdgeDirection::Backward,
            car_average_speed,
        );
    }
}
