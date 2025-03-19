use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue};
use hermes_core::geopoint::GeoPoint;
use hermes_core::hermes::Hermes;
use hermes_core::routing::routing_request::RoutingRequest;
use std::fs;

fn main() {
    let hermes = Hermes::new_from_osm("./data/osm/brussels_capital_region-latest.osm.pbf");

    let avenue_louise = GeoPoint {
        lat: 50.822147,
        lng: 4.369564,
    };

    let rue_des_palais = GeoPoint {
        lat: 50.866,
        lng: 4.3662,
    };

    let path = hermes
        .route(RoutingRequest {
            start: avenue_louise,
            end: rue_des_palais,
            profile: "car",
        })
        .unwrap();

    let mut features: Vec<geojson::Feature> = Vec::new();

    for leg in path.legs() {
        println!("Leg distance {}", leg.distance());
        println!("Leg time {}", leg.time());

        let mut properties = JsonObject::new();

        properties.insert("stroke".to_string(), JsonValue::from("blue"));
        properties.insert("stroke-width".to_string(), JsonValue::from(1.0));

        features.push(Feature {
            bbox: None,
            id: None,
            properties: Some(properties),
            foreign_members: None,
            geometry: Some(Geometry::new(LineString(
                leg.points()
                    .iter()
                    .map(|coordinates| vec![coordinates.lng, coordinates.lat])
                    .collect(),
            ))),
        })
    }

    let geojson = GeoJson::FeatureCollection(FeatureCollection {
        bbox: None,
        foreign_members: None,
        features,
    });
    let geojson_string = geojson.to_string();
    let result = fs::write("./data/geojson/path.geojson", geojson_string);
    match result {
        Ok(_) => println!("GeoJSON file written successfully."),
        Err(e) => println!("Error writing GeoJSON file: {}", e),
    }
}
