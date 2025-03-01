use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property::VehicleAccess;
use crate::properties::property_map::{BACKWARD_EDGE, EdgePropertyMap, FORWARD_EDGE};
use crate::properties::tag_parser::TagParser;

static HIGHWAY_VALUES: [&str; 16] = [
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

fn get_car_access(way: &OsmWay) -> WayAccess {
    let highway = way.get_tag("highway");

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
    way.get_tag("oneway")
        .map_or(false, |value| ONEWAYS.contains(&value))
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
    fn handle_way(way: &OsmWay, properties: &mut EdgePropertyMap) {
        let vehicleKey = "car";
        if let WayAccess::Way = get_car_access(way) {
            if is_oneway(way) || is_roundabout(way) {
                if is_forward_oneway(way) {
                    properties.insert_bool(
                        VehicleAccess(vehicleKey.to_string()),
                        FORWARD_EDGE,
                        true,
                    );
                }

                if is_backward_oneway(way) {
                    properties.insert_bool(
                        VehicleAccess(vehicleKey.to_string()),
                        BACKWARD_EDGE,
                        true,
                    );
                }
            } else {
                properties.insert_bool(VehicleAccess(vehicleKey.to_string()), FORWARD_EDGE, true);
                properties.insert_bool(VehicleAccess(vehicleKey.to_string()), BACKWARD_EDGE, true);
            }
        } else {
            properties.insert_bool(VehicleAccess(vehicleKey.to_string()), FORWARD_EDGE, false);
            properties.insert_bool(VehicleAccess(vehicleKey.to_string()), BACKWARD_EDGE, false);
        }
    }
}

enum WayAccess {
    Way,
    None,
}
