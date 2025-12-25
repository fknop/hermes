use crate::constants::MPH_TO_KPH;
use crate::edge_direction::EdgeDirection;
use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property;
use crate::properties::tag_parser::TagParser;

use super::property_map::EdgePropertyMap;

pub struct MaxSpeedParser;

impl MaxSpeedParser {
    fn remove_non_numeric_chars(speed_tag: &str) -> String {
        speed_tag.chars().filter(|c| c.is_ascii_digit()).collect()
    }

    fn parse_max_speed_tag(max_speed_tag: Option<&str>) -> Option<f32> {
        match max_speed_tag {
            Some("walk") => Some(5.0),
            Some("none") => Some(150.0),
            Some(max_speed) => {
                if max_speed.contains("mph") {
                    let raw_mph = MaxSpeedParser::remove_non_numeric_chars(max_speed);
                    raw_mph.parse::<f32>().ok().map(|mph| mph * MPH_TO_KPH)
                } else if max_speed.contains("kph") {
                    let raw_kph = MaxSpeedParser::remove_non_numeric_chars(max_speed);
                    raw_kph.parse::<f32>().ok()
                } else {
                    max_speed.parse::<f32>().ok()
                }
            }
            None => None,
        }
    }

    pub fn parse_max_speed(way: &OsmWay) -> Option<f32> {
        MaxSpeedParser::parse_max_speed_tag(way.tag("maxspeed"))
    }
}

// https://wiki.openstreetmap.org/wiki/Key:maxspeed
impl TagParser for MaxSpeedParser {
    fn parse_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        if let Some(max_speed) = MaxSpeedParser::parse_max_speed(way) {
            properties.insert_f32(Property::MaxSpeed, EdgeDirection::Forward, max_speed);
            properties.insert_f32(Property::MaxSpeed, EdgeDirection::Backward, max_speed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 0.001;

    #[test]
    fn test_parse_max_speed() {
        assert!((MaxSpeedParser::parse_max_speed_tag(Some("50")).unwrap() - 50.0).abs() < EPSILON);
        assert!(
            (MaxSpeedParser::parse_max_speed_tag(Some("50 mph")).unwrap() - 80.467).abs() < EPSILON
        );
        assert!(
            (MaxSpeedParser::parse_max_speed_tag(Some("50mph")).unwrap() - 80.467).abs() < EPSILON
        );
        assert!(
            (MaxSpeedParser::parse_max_speed_tag(Some("50 kph")).unwrap() - 50.0).abs() < EPSILON
        );
        assert!(
            (MaxSpeedParser::parse_max_speed_tag(Some("50kph")).unwrap() - 50.0).abs() < EPSILON
        );
        assert!((MaxSpeedParser::parse_max_speed_tag(Some("walk")).unwrap() - 5.0).abs() < EPSILON);
        assert!(
            (MaxSpeedParser::parse_max_speed_tag(Some("none")).unwrap() - 150.0).abs() < EPSILON
        );
        assert_eq!(MaxSpeedParser::parse_max_speed_tag(None), None);
    }
}
