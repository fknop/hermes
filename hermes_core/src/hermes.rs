use crate::base_graph::BaseGraph;
use crate::geopoint::GeoPoint;
use crate::location_index::LocationIndex;
use crate::query_graph::QueryGraph;
use crate::routing::astar::AStar;
use crate::routing::bidirectional_astar::BidirectionalAStar;
use crate::routing::dijkstra::Dijkstra;
use crate::routing::routing_request::{RoutingAlgorithm, RoutingRequest};

use crate::routing::shortest_path_algorithm::{
    ShortestPathAlgorithm, ShortestPathOptions, ShortestPathResult,
};
use crate::stopwatch::Stopwatch;
use crate::weighting::{CarWeighting, Weighting};
use std::collections::HashMap;
use std::path::Path;

pub struct Hermes {
    graph: BaseGraph,
    index: LocationIndex,
    // TODO: Sync + Send, I don't know what I'm doing here
    // profiles: HashMap<String, Box<dyn Weighting + Sync + Send>>,
    car_weighting: CarWeighting,
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
        let graph =
            BaseGraph::from_file(graph_file.into_os_string().into_string().unwrap().as_str());
        let location_index = LocationIndex::build_from_graph(&graph);

        // let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        // profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index: location_index,
            car_weighting: CarWeighting::new(), // profiles,
        }
    }

    pub fn from_osm_file(file_path: &str) -> Hermes {
        let graph = BaseGraph::from_osm_file(file_path);
        let index = LocationIndex::build_from_graph(&graph);

        let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index,
            car_weighting: CarWeighting::new(), // profiles,
        }
    }

    pub fn graph(&self) -> &BaseGraph {
        &self.graph
    }

    pub fn index(&self) -> &LocationIndex {
        &self.index
    }

    pub fn route(&self, request: RoutingRequest) -> Result<ShortestPathResult, String> {
        let weighting = match request.profile.as_str() {
            "car" => &self.car_weighting,
            _ => return Err(String::from("No profile found")),
        };

        let start_snap_stop_watch = Stopwatch::new("snap/start");
        let start_snap = self
            .index()
            .snap(&self.graph, weighting, &request.start)
            .expect("no way to avenue closest way");

        start_snap_stop_watch.report();

        let end_snap_stop_watch = Stopwatch::new("snap/end");

        let end_snap = self
            .index()
            .snap(&self.graph, weighting, &request.end)
            .expect("no way to rue des palais way");

        end_snap_stop_watch.report();

        let mut snaps = [start_snap, end_snap];

        let build_query_graph_sw = Stopwatch::new("querygraph/build");

        let query_graph = QueryGraph::from_base_graph(&self.graph, &mut snaps[..]);

        build_query_graph_sw.report();

        let start = snaps[0].closest_node();
        let end = snaps[1].closest_node();

        let request_options = request.options.as_ref();
        let options = ShortestPathOptions {
            include_debug_info: request_options.and_then(|options| options.include_debug_info),
        };

        match request_options.and_then(|options| options.algorithm) {
            Some(RoutingAlgorithm::Dijkstra) => {
                let mut dijkstra = Dijkstra::new(&query_graph);
                dijkstra.calc_path(&query_graph, weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::Astar) => {
                let mut astar = AStar::new(&query_graph);
                astar.calc_path(&query_graph, weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::BidirectionalAstar) => {
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&query_graph, weighting, start, end, Some(options))
            }
            None => {
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&query_graph, weighting, start, end, Some(options))
            }
        }
    }

    pub fn closest_edge(&self, profile: String, coordinates: GeoPoint) -> Option<usize> {
        let snap = self
            .index
            .snap(self.graph(), &self.car_weighting, &coordinates)?;
        Some(snap.edge_id)
    }
}
