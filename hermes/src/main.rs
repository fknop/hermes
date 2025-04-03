use std::fs;

use geojson::{Feature, Value};
use hermes_core::hermes::Hermes;

fn main() {
    let hermes = Hermes::from_osm_file("./data/osm/belgium-latest.osm.pbf");
    hermes.save("./data/");

    let landmarks = hermes.create_landmarks();

    let points = landmarks
        .iter()
        .map(|point| vec![point.lon(), point.lat()])
        .collect();

    let feature = Feature {
        geometry: Some(geojson::Geometry::new(Value::MultiPoint(points))),
        ..Default::default()
    };

    fs::write("./data/landmarks.geojson", feature.to_string());
    ()
}
