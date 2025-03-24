use crate::osm::osm_reader::OsmWay;
use crate::properties::property::Property::VehicleAccess;
use crate::properties::property_map::{BACKWARD_EDGE, FORWARD_EDGE};
use crate::properties::tag_parser::TagParser;

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
    fn handle_way(way: &mut OsmWay) {
        let vehicle_key = "car";
        if let WayAccess::Way = car_access(way) {
            if is_oneway(way) || is_roundabout(way) {
                if is_forward_oneway(way) {
                    way.properties_mut().insert_bool(
                        VehicleAccess(vehicle_key.to_string()),
                        FORWARD_EDGE,
                        true,
                    );
                }

                if is_backward_oneway(way) {
                    way.properties_mut().insert_bool(
                        VehicleAccess(vehicle_key.to_string()),
                        BACKWARD_EDGE,
                        true,
                    );
                }
            } else {
                way.properties_mut().insert_bool(
                    VehicleAccess(vehicle_key.to_string()),
                    FORWARD_EDGE,
                    true,
                );
                way.properties_mut().insert_bool(
                    VehicleAccess(vehicle_key.to_string()),
                    BACKWARD_EDGE,
                    true,
                );
            }
        } else {
            way.properties_mut().insert_bool(
                VehicleAccess(vehicle_key.to_string()),
                FORWARD_EDGE,
                false,
            );
            way.properties_mut().insert_bool(
                VehicleAccess(vehicle_key.to_string()),
                BACKWARD_EDGE,
                false,
            );
        }
    }
}

enum WayAccess {
    Way,
    None,
}
