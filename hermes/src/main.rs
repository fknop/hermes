use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue};
use hermes_core::geopoint::GeoPoint;
use hermes_core::hermes::Hermes;
use hermes_core::routing::routing_request::RoutingRequest;
use std::fs;

fn main() {
    let hermes = Hermes::from_directory("./data");
    println!("Hermes node count {}", hermes.graph().node_count());
    println!("Hermes edge count {}", hermes.graph().edge_count());
    // let hermes = Hermes::from_osm_file("./data/osm/brussels_capital_region-latest.osm.pbf");
    // hermes.save("./data/");
}
