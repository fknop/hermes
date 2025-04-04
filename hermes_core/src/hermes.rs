use crate::base_graph::BaseGraph;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::landmarks::landmarks_astar::LandmarksAstar;
use crate::landmarks::landmarks_data::LandmarksData;
use crate::landmarks::landmarks_preparation::LandmarksPreparation;
use crate::location_index::LocationIndex;
use crate::query_graph::QueryGraph;
use crate::routing::astar::AStar;
use crate::routing::bidirectional_astar::BidirectionalAStar;
use crate::routing::dijkstra::Dijkstra;
use crate::routing::routing_request::{RoutingAlgorithm, RoutingRequest};

use crate::routing::shortest_path_algorithm::{CalcPath, CalcPathOptions, CalcPathResult};
use crate::stopwatch::Stopwatch;
use crate::storage::binary_file_path;
use crate::weighting::CarWeighting;

pub struct Hermes {
    graph: BaseGraph,
    index: LocationIndex,
    // TODO: Sync + Send, I don't know what I'm doing here
    // profiles: HashMap<String, Box<dyn Weighting + Sync + Send>>,
    car_weighting: CarWeighting,

    lm: LandmarksData,
}

const GRAPH_FILE_NAME: &str = "graph.bin";
const LANDMARKS_FILE_NAME: &str = "lm.bin";

impl Hermes {
    pub fn save(&self, dir_path: &str) {
        self.graph
            .save_to_file(binary_file_path(dir_path, GRAPH_FILE_NAME).as_str());

        self.lm
            .save_to_file(binary_file_path(dir_path, LANDMARKS_FILE_NAME).as_str());
    }

    pub fn from_directory(dir_path: &str) -> Hermes {
        let graph = BaseGraph::from_file(binary_file_path(dir_path, GRAPH_FILE_NAME).as_str());
        let location_index = LocationIndex::build_from_graph(&graph);

        let lm = LandmarksData::from_file(binary_file_path(dir_path, LANDMARKS_FILE_NAME).as_str());

        // let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        // profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index: location_index,
            car_weighting: CarWeighting::new(), // profiles,
            lm,
        }
    }

    pub fn from_osm_file(file_path: &str) -> Hermes {
        let graph = BaseGraph::from_osm_file(file_path);

        // let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // // Add default profile
        // profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        let weighting = CarWeighting::new();
        let lm_preparation = LandmarksPreparation::new(&graph, &weighting);
        let lm = lm_preparation.create_landmarks(16);

        let index = LocationIndex::build_from_graph(&graph);

        Hermes {
            graph,
            index,
            car_weighting: weighting, // profiles,
            lm,
        }
    }

    pub fn graph(&self) -> &BaseGraph {
        &self.graph
    }

    pub fn index(&self) -> &LocationIndex {
        &self.index
    }

    pub fn route(&self, request: RoutingRequest) -> Result<CalcPathResult, String> {
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
        let options = CalcPathOptions {
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

            Some(RoutingAlgorithm::Landmarks) => {
                let mut landmarks_astar =
                    LandmarksAstar::new(&query_graph, weighting, &self.lm, start, end);
                landmarks_astar.calc_path(&query_graph, weighting, start, end, Some(options))
            }

            None => {
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&query_graph, weighting, start, end, Some(options))
            }
        }
    }

    pub fn create_landmarks(&self) -> Vec<GeoPoint> {
        let lm_preparation = LandmarksPreparation::new(self.graph(), &self.car_weighting);

        let landmarks = lm_preparation.find_landmarks(10);
        landmarks
            .iter()
            .map(|node| *self.graph.node_geometry(*node))
            .collect()
    }

    pub fn closest_edge(&self, profile: String, coordinates: GeoPoint) -> Option<usize> {
        let snap = self
            .index
            .snap(self.graph(), &self.car_weighting, &coordinates)?;
        Some(snap.edge_id)
    }
}
