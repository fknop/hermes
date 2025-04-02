use crate::edge_direction::EdgeDirection;
use crate::osm::osm_reader::OsmWay;
use crate::properties::tag_parser::TagParser;

use super::property::Property;
use super::property_map::EdgePropertyMap;

pub static HIGHWAY_VALUES: [&str; 16] = [
    "motorway",
    "motorway_link",
    "trunk",
    "trunk_link",
    "primary",
    "primary_link",
    "secondary",
    "secondary_link",
    "tertiary",
    "tertiary_link",
    "unclassified",
    "residential",
    "living_street",
    "service",
    "road",
    "track",
];

static ONEWAYS: [&str; 4] = ["yes", "true", "1", "-1"];

pub struct CarAccessParser;

fn car_access(way: &OsmWay) -> WayAccess {
    let highway = way.tag("highway");

    if highway.is_none() {
        return WayAccess::None;
    }

    match highway {
        // https://wiki.openstreetmap.org/wiki/Tag:highway%3Dservice
        Some("service") if way.has_tag("service", "emergency_access") => WayAccess::None,
        Some(value) if HIGHWAY_VALUES.contains(&value) => WayAccess::Way,
        _ => WayAccess::None,
    }
}

// https://wiki.openstreetmap.org/wiki/Key:oneway
fn is_oneway(way: &OsmWay) -> bool {
    way.tag("oneway")
        .is_some_and(|value| ONEWAYS.contains(&value))
}

fn is_forward_oneway(way: &OsmWay) -> bool {
    !way.has_tag("oneway", "-1")
}

fn is_backward_oneway(way: &OsmWay) -> bool {
    way.has_tag("oneway", "-1")
}

// https://wiki.openstreetmap.org/wiki/Key:junction
fn is_roundabout(way: &OsmWay) -> bool {
    way.has_tag("junction", "roundabout") || way.has_tag("junction", "circular")
}

// https://wiki.openstreetmap.org/wiki/Tag:highway%3Dservice
impl TagParser for CarAccessParser {
    fn parse_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        if let WayAccess::Way = car_access(way) {
            if is_oneway(way) || is_roundabout(way) {
                if is_forward_oneway(way) {
                    properties.insert_bool(
                        Property::CarVehicleAccess,
                        EdgeDirection::Forward,
                        true,
                    );
                }

                if is_backward_oneway(way) {
                    properties.insert_bool(
                        Property::CarVehicleAccess,
                        EdgeDirection::Backward,
                        true,
                    );
                }
            } else {
                properties.insert_bool(Property::CarVehicleAccess, EdgeDirection::Forward, true);
                properties.insert_bool(Property::CarVehicleAccess, EdgeDirection::Backward, true);
            }
        } else {
            properties.insert_bool(Property::CarVehicleAccess, EdgeDirection::Forward, false);
            properties.insert_bool(Property::CarVehicleAccess, EdgeDirection::Backward, false);
        }
    }
}

enum WayAccess {
    Way,
    None,
}
