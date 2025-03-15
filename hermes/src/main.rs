use geojson::Value::LineString;
use geojson::{Feature, FeatureCollection, GeoJson, Geometry, JsonObject, JsonValue};
use hermes_core::dijkstra::{Dijkstra, ShortestPathAlgo};
use hermes_core::graph::Graph;
use hermes_core::latlng::LatLng;
use hermes_core::location_index::LocationIndex;
use hermes_core::osm::osm_reader::parse_osm_file;
use hermes_core::weighting::CarWeighting;
use std::fs;

fn main() {
    let osm_data = parse_osm_file("./data/osm/brussels_capital_region-latest.osm.pbf");

    println!("node_data len {}", osm_data.get_nodes().len());
    println!("way_data len {}", osm_data.get_ways().len());

    let graph = Graph::build_from_osm_data(&osm_data);
    let index = LocationIndex::build_from_graph(&graph);

    println!("edge count {}", graph.get_edge_count());

    let avenue_louise = LatLng {
        lat: 50.822147,
        lng: 4.369564,
    };

    let rue_des_palais = LatLng {
        lat: 50.866,
        lng: 4.3662,
    };

    let closest1 = index
        .get_closest(&rue_des_palais)
        .expect("no way to avenue closest way");
    let closest2 = index
        .get_closest(&avenue_louise)
        .expect("no way to rue des palais way");

    let from = graph.get_edge(closest1).get_from_node();
    let to = graph.get_edge(closest2).get_to_node();

    println!("edges for from {}", graph.get_node_edges(from).len());
    println!("edges for to {}", graph.get_node_edges(to).len());

    let mut dijkstra = Dijkstra::new(&graph);

    let weighting = CarWeighting::new();
    let path = dijkstra.calc_path(&graph, &weighting, from, to);

    let mut features: Vec<geojson::Feature> = Vec::new();

    for leg in path.get_legs() {
        println!("Leg distance {}", leg.get_distance());
        println!("Leg time {}", leg.get_time());

        let mut properties = JsonObject::new();

        properties.insert("stroke".to_string(), JsonValue::from("blue"));
        properties.insert("stroke-width".to_string(), JsonValue::from(1.0));

        features.push(Feature {
            bbox: None,
            id: None,
            properties: Some(properties),
            foreign_members: None,
            geometry: Some(Geometry::new(LineString(
                leg.get_points()
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
