use crate::base_graph::BaseGraph;
use crate::error::ImportError;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::landmarks::lm_bidirectional_astar::LMBidirectionalAstar;
use crate::landmarks::lm_data::LMData;
use crate::landmarks::lm_preparation::LMPreparation;
use crate::location_index::LocationIndex;
use crate::query_graph::QueryGraph;
use crate::routing::astar::AStar;
use crate::routing::bidirectional_astar::BidirectionalAStar;
use crate::routing::dijkstra::Dijkstra;
use crate::routing::routing_request::{RoutingAlgorithm, RoutingRequest};

use crate::routing::shortest_path_algorithm::{CalcPath, CalcPathOptions, CalcPathResult};
use crate::stopwatch::Stopwatch;
use crate::storage::binary_file_path;
use crate::weighting::{CarWeighting, Weighting};

pub struct Hermes {
    graph: BaseGraph,
    index: LocationIndex,
    // TODO: Sync + Send, I don't know what I'm doing here
    // profiles: HashMap<String, Box<dyn Weighting + Sync + Send>>,
    // car_weighting: CarWeighting<QueryGraph<'a>>,
    lm: LMData,
}

const GRAPH_FILE_NAME: &str = "graph.bin";
const LANDMARKS_FILE_NAME: &str = "lm.bin";
const LOCATION_INDEX_FILE_NAME: &str = "location_index.bin";

impl Hermes {
    pub fn save(&self, dir_path: &str) -> Result<(), ImportError> {
        self.graph
            .save_to_file(binary_file_path(dir_path, GRAPH_FILE_NAME).as_str())
            .map_err(ImportError::SaveGraph)?;

        self.lm
            .save_to_file(binary_file_path(dir_path, LANDMARKS_FILE_NAME).as_str())
            .map_err(ImportError::SaveLandmarks)?;

        self.index
            .save_to_file(binary_file_path(dir_path, LOCATION_INDEX_FILE_NAME).as_str())
            .map_err(ImportError::SaveLocationIndex)?;

        Ok(())
    }

    pub fn from_directory(dir_path: &str) -> Hermes {
        let graph = BaseGraph::from_file(binary_file_path(dir_path, GRAPH_FILE_NAME).as_str());
        let location_index = LocationIndex::load_from_file(
            binary_file_path(dir_path, LOCATION_INDEX_FILE_NAME).as_str(),
        );

        let lm = LMData::from_file(binary_file_path(dir_path, LANDMARKS_FILE_NAME).as_str());

        // let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // Add default profile
        // profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        Hermes {
            graph,
            index: location_index,
            lm,
        }
    }

    pub fn from_osm_file(file_path: &str) -> Hermes {
        let graph = BaseGraph::from_osm_file(file_path);

        // let mut profiles: HashMap<String, Box<dyn Weighting + Sync + Send>> = HashMap::new();
        // // Add default profile
        // profiles.insert("car".to_string(), Box::from(CarWeighting::new()));

        let weighting = CarWeighting::new();
        let lm_preparation = LMPreparation::new(&graph, &weighting);
        let lm = lm_preparation.create_landmarks(10);

        let index = LocationIndex::build_from_graph(&graph);

        Hermes { graph, index, lm }
    }

    pub fn graph(&self) -> &BaseGraph {
        &self.graph
    }

    pub fn index(&self) -> &LocationIndex {
        &self.index
    }

    pub fn route(&self, request: RoutingRequest) -> Result<CalcPathResult, String> {
        let base_graph_weighting = self.create_weighting(&request.profile);

        let start_snap_stop_watch = Stopwatch::new("snap/start");
        let start_snap = self
            .index()
            .snap(&self.graph, &base_graph_weighting, &request.start)
            .expect("no way to avenue closest way");

        start_snap_stop_watch.report();

        let end_snap_stop_watch = Stopwatch::new("snap/end");

        let end_snap = self
            .index()
            .snap(&self.graph, &base_graph_weighting, &request.end)
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
                let weighting = self.create_weighting(&request.profile);
                let mut dijkstra = Dijkstra::new(&query_graph);
                dijkstra.calc_path(&query_graph, &weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::Astar) => {
                let weighting = self.create_weighting(&request.profile);
                let mut astar = AStar::new(&query_graph);
                astar.calc_path(&query_graph, &weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::BidirectionalAstar) => {
                let weighting = self.create_weighting(&request.profile);
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&query_graph, &weighting, start, end, Some(options))
            }

            Some(RoutingAlgorithm::Landmarks) => {
                let weighting = self.create_weighting(&request.profile);
                let mut landmarks_astar = LMBidirectionalAstar::from_landmarks(
                    &query_graph,
                    &weighting,
                    &self.lm,
                    start,
                    end,
                );
                landmarks_astar.calc_path(&query_graph, &weighting, start, end, Some(options))
            }

            None => {
                let weighting = self.create_weighting(&request.profile);
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&query_graph, &weighting, start, end, Some(options))
            }
        }
    }

    fn create_weighting<G: Graph>(&self, profile: &str) -> impl Weighting<G> {
        match profile {
            "car" => CarWeighting::new(),
            _ => panic!("No profile found"),
        }
    }
}
