use crate::dijkstra::{Dijkstra, ShortestPathAlgo};
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::location_index::LocationIndex;
use crate::osm::osm_reader::parse_osm_file;
use crate::routing::routing_request::RoutingRequest;
use crate::routing_path::RoutingPath;
use crate::weighting::{CarWeighting, Weighting};
use std::collections::HashMap;
use std::path::Path;

pub struct Hermes {
    graph: Graph,
    index: LocationIndex,
    // TODO: Sync + Send, I don't know what I'm doing here
    profiles: HashMap<String, Box<dyn Weighting + Sync + Send>>,
}

const GRAPH_FILE_NAME: &str = "graph.bin";

impl Hermes {
    pub fn save(&self, dir_path: &str) {
        let directory = Path::new(dir_path);
        let graph_file = directory.join(GRAPH_FILE_NAME);

        self.graph()
            .save_to_file(graph_file.into_os_string().into_string().unwrap().as_str());
    }

    pub fn from_directory(dir_path: &str) -> Hermes {
        let directory = Path::new(dir_path);
        let graph_file = directory.join(GRAPH_FILE_NAME);
        let graph = Graph::from_file(graph_file.into_os_string().into_string().unwrap().as_str());
        let location_index = LocationIndex::build_from_graph(&graph);

        let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index: location_index,
            profiles,
        }
    }

    pub fn from_osm_file(file_path: &str) -> Hermes {
        let osm_data = parse_osm_file(file_path);

        let graph = Graph::from_osm_data(&osm_data);
        let index = LocationIndex::build_from_graph(&graph);

        let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index,
            profiles,
        }
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn index(&self) -> &LocationIndex {
        &self.index
    }

    pub fn route(&self, request: RoutingRequest) -> Result<RoutingPath, String> {
        let profile = self.profiles.get(&request.profile);

        if profile.is_none() {
            return Err(format!("No profile found for {}", request.profile));
        }

        let weighting = profile.unwrap().as_ref();

        let mut dijkstra = Dijkstra::new(self.graph());

        let start_snap = self
            .index()
            .snap(&self.graph, weighting, &request.start)
            .expect("no way to avenue closest way");
        let end_snap = self
            .index()
            .snap(&self.graph, weighting, &request.end)
            .expect("no way to rue des palais way");

        println!("start_snap {:?}", start_snap);
        println!("end_snap {:?}", end_snap);

        let start = self.graph().edge(start_snap.edge_id).start_node();
        let end = self.graph().edge(end_snap.edge_id).end_node();

        let path = dijkstra.calc_path(self.graph(), weighting, start, end);
        Ok(path)
    }

    pub fn closest_edge(&self, profile: String, coordinates: GeoPoint) -> Option<usize> {
        let weighting = self.profiles.get(&profile)?;

        let snap = self
            .index
            .snap(self.graph(), weighting.as_ref(), &coordinates)?;
        Some(snap.edge_id)
    }
}
