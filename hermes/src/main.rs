use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue};
use hermes_core::graph::Graph;
use hermes_core::latlng::LatLng;
use hermes_core::location_index::LocationIndex;
use hermes_core::osm::osm_reader::parse_osm_file;
use std::fs;

fn main() {
    let osm_data = parse_osm_file("./data/osm/brussels_capital_region-latest.osm.pbf");

    println!("node_data len {}", osm_data.get_nodes().len());
    println!("way_data len {}", osm_data.get_ways().len());

    let graph = Graph::build_from_osm_data(&osm_data);
    let index = LocationIndex::build_from_graph(&graph);

    println!("edge count {}", graph.get_edge_count());

    let avenueLouise = LatLng {
        lat: 50.822147,
        lng: 4.369564,
    };

    let closest = index.get_closest(&avenueLouise);

    if let Some(closest) = closest {
        let edge = graph.get_edge(closest);
        let geometry = graph.get_edge_geometry(closest);
        geometry
            .iter()
            .for_each(|c| println!("lat {}, lng {}", c.lat, c.lng));
    }

    //
    // let mut features: Vec<geojson::Feature> = Vec::new();
    //
    // osm_data.get_ways().iter().for_each(|way| {
    //     let positions = way
    //         .nodes
    //         .iter()
    //         .map(|node| {
    //             let node = osm_data.get_node(*node).unwrap();
    //             vec![node.coordinates.lng, node.coordinates.lat]
    //         })
    //         .collect::<Vec<Vec<f64>>>();
    //
    //     let mut properties = JsonObject::new();
    //
    //     properties.insert("stroke".to_string(), JsonValue::from("blue"));
    //     properties.insert("stroke-width".to_string(), JsonValue::from(1.0));
    //
    //     way.tags.iter().for_each(|tag| {
    //         properties.insert(tag.0.clone(), JsonValue::from(tag.1.clone()));
    //     });
    //
    //     features.push(Feature {
    //         bbox: None,
    //         id: None,
    //         properties: Some(properties),
    //         foreign_members: None,
    //         geometry: Some(Geometry::new(LineString(positions))),
    //     })
    // });
    //
    // let geojson = GeoJson::FeatureCollection(FeatureCollection {
    //     bbox: None,
    //     foreign_members: None,
    //     features,
    // });
    //
    // let geojson_string = geojson.to_string();
    //
    // let result = fs::write(
    //     "./data/geojson/brussels_capital_region.geojson",
    //     geojson_string,
    // );
    //
    // match result {
    //     Ok(_) => println!("GeoJSON file written successfully."),
    //     Err(e) => println!("Error writing GeoJSON file: {}", e),
    // }
}
