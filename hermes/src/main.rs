use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue};
use hermes_core::osm::osm_reader::parse_osm_file;
use std::fs;

fn main() {
    let osm_data = parse_osm_file("./data/osm/brussels_capital_region-latest.osm.pbf");

    println!("node_data len {}", osm_data.osm_node_data.len());
    println!("way_data len {}", osm_data.osm_ways_data.len());

    let mut features: Vec<geojson::Feature> = Vec::new();

    osm_data.get_ways().iter().for_each(|way| {
        let positions = way
            .nodes
            .iter()
            .map(|node| {
                let node = osm_data.get_node(*node).unwrap();
                vec![node.coordinates.lng, node.coordinates.lat]
            })
            .collect::<Vec<Vec<f64>>>();

        let mut properties = JsonObject::new();

        properties.insert("stroke".to_string(), JsonValue::from("blue"));
        properties.insert("stroke-width".to_string(), JsonValue::from(1.0));

        way.tags.iter().for_each(|tag| {
            properties.insert(tag.0.clone(), JsonValue::from(tag.1.clone()));
        });

        features.push(Feature {
            bbox: None,
            id: None,
            properties: Some(properties),
            foreign_members: None,
            geometry: Some(Geometry::new(LineString(positions))),
        })
    });

    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        foreign_members: None,
        features,
    });

    let geojson_string = geojson.to_string();

    let result = fs::write(
        "./data/geojson/brussels_capital_region.geojson",
        geojson_string,
    );

    match result {
        Ok(_) => println!("GeoJSON file written successfully."),
        Err(e) => println!("Error writing GeoJSON file: {}", e),
    }
}
