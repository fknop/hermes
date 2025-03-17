use crate::dijkstra::{Dijkstra, ShortestPathAlgo};
use crate::graph::Graph;
use crate::location_index::LocationIndex;
use crate::osm::osm_reader::parse_osm_file;
use crate::routing::routing_request::RoutingRequest;
use crate::routing_path::RoutingPath;
use crate::weighting::{CarWeighting, Weighting};
use std::collections::HashMap;

pub struct Hermes {
    graph: Graph,
    index: LocationIndex,
    // TODO: Sync + Send, I don't know what I'm doing here
    profiles: HashMap<String, Box<dyn Weighting + Sync + Send>>,
}

impl Hermes {
    pub fn new_from_osm(file_path: &str) -> Hermes {
        let osm_data = parse_osm_file(file_path);

        let graph = Graph::build_from_osm_data(&osm_data);
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

        let mut dijkstra = Dijkstra::new(self.graph());

        let start_snap = self
            .index()
            .closest(&request.start)
            .expect("no way to avenue closest way");
        let end_snap = self
            .index()
            .closest(&request.end)
            .expect("no way to rue des palais way");

        let start = self.graph().edge(start_snap).start_node();
        let end = self.graph().edge(end_snap).end_node();

        let weighting = profile.unwrap().as_ref();

        let path = dijkstra.calc_path(self.graph(), weighting, start, end);
        Ok(path)
    }
}
