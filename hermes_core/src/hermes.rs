use crate::base_graph::BaseGraph;
use crate::ch::ch_graph::CHGraph;
use crate::ch::ch_graph_builder::CHGraphBuilder;
use crate::ch::ch_storage::CHStorage;
use crate::ch::ch_weighting::CHWeighting;
use crate::error::ImportError;
use crate::geopoint::GeoPoint;
use crate::graph::{GeometryAccess, Graph};
use crate::landmarks::lm_bidirectional_astar::LMBidirectionalAstar;
use crate::landmarks::lm_data::LMData;
use crate::landmarks::lm_preparation::LMPreparation;
use crate::location_index::LocationIndex;
use crate::query::query_graph::QueryGraph;
use crate::routing::astar::AStar;
use crate::routing::bidirectional_astar::BidirectionalAStar;
use crate::routing::ch_bidirectional_dijkstra::{
    self, CHBidirectionalAStar, CHBidirectionalDijkstra, CHLMAstar,
};
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
    ch_storage: Option<CHStorage>,
}

const GRAPH_FILE_NAME: &str = "graph.bin";
const LANDMARKS_FILE_NAME: &str = "lm.bin";
const LOCATION_INDEX_FILE_NAME: &str = "location_index.bin";
const CH_GRAPH_FILE_NAME: &str = "ch_graph.bin";

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

        if let Some(ch_storage) = &self.ch_storage {
            ch_storage
                .save_to_file(binary_file_path(dir_path, CH_GRAPH_FILE_NAME).as_str())
                .map_err(ImportError::SaveCHGraph)?;
        }

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

        let ch_storage =
            CHStorage::from_file(binary_file_path(dir_path, CH_GRAPH_FILE_NAME).as_str());

        Hermes {
            graph,
            index: location_index,
            lm,
            ch_storage: Some(ch_storage),
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

        let mut ch_builder = CHGraphBuilder::from_base_graph(&graph);
        let ch_storage = ch_builder.build(&weighting);

        Hermes {
            graph,
            index,
            lm,
            ch_storage: Some(ch_storage),
        }
    }

    pub fn graph(&self) -> &BaseGraph {
        &self.graph
    }

    pub fn index(&self) -> &LocationIndex {
        &self.index
    }

    pub fn get_landmarks(&self) -> Vec<GeoPoint> {
        self.lm
            .get_node_ids()
            .iter()
            .map(|&node_id| self.graph.node_geometry(node_id))
            .copied()
            .collect()
    }

    pub fn route(&self, request: RoutingRequest) -> Result<CalcPathResult, String> {
        let base_graph_weighting = self.create_weighting(&request.profile);

        let start_snap = self
            .index()
            .snap(&self.graph, &base_graph_weighting, &request.start)
            .expect("no way to avenue closest way");

        let end_snap = self
            .index()
            .snap(&self.graph, &base_graph_weighting, &request.end)
            .expect("no way to rue des palais way");

        let mut snaps = [start_snap, end_snap];

        let request_options = request.options.as_ref();
        let options = CalcPathOptions {
            include_debug_info: request_options.and_then(|options| options.include_debug_info),
        };

        match request_options.and_then(|options| options.algorithm) {
            Some(RoutingAlgorithm::Dijkstra) => {
                let weighting = self.create_weighting(&request.profile);
                let query_graph = QueryGraph::from_graph(&self.graph, &self.graph, &mut snaps[..]);
                let start = snaps[0].closest_node();
                let end = snaps[1].closest_node();

                let mut dijkstra = Dijkstra::new(&query_graph);
                dijkstra.calc_path(&weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::Astar) => {
                let weighting = self.create_weighting(&request.profile);
                let query_graph = QueryGraph::from_graph(&self.graph, &self.graph, &mut snaps[..]);
                let start = snaps[0].closest_node();
                let end = snaps[1].closest_node();

                let mut astar = AStar::new(&query_graph);
                astar.calc_path(&weighting, start, end, Some(options))
            }
            Some(RoutingAlgorithm::BidirectionalAstar) => {
                let weighting = self.create_weighting(&request.profile);
                let query_graph = QueryGraph::from_graph(&self.graph, &self.graph, &mut snaps[..]);
                let start = snaps[0].closest_node();
                let end = snaps[1].closest_node();
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&weighting, start, end, Some(options))
            }

            Some(RoutingAlgorithm::Landmarks) => {
                let weighting = self.create_weighting(&request.profile);
                let query_graph = QueryGraph::from_graph(&self.graph, &self.graph, &mut snaps[..]);
                let start = snaps[0].closest_node();
                let end = snaps[1].closest_node();
                let mut landmarks_astar = LMBidirectionalAstar::from_landmarks(
                    &query_graph,
                    &weighting,
                    &self.lm,
                    start,
                    end,
                );
                landmarks_astar.calc_path(&weighting, start, end, Some(options))
            }

            Some(RoutingAlgorithm::ContractionHierarchies) => match &self.ch_storage {
                Some(ch_storage) => {
                    let weighting = CHWeighting::new();

                    let ch_graph = CHGraph::new(ch_storage, &self.graph);

                    let query_graph =
                        QueryGraph::from_graph(&ch_graph, &self.graph, &mut snaps[..]);
                    let start = snaps[0].closest_node();
                    let end = snaps[1].closest_node();

                    // let mut ch_bidirectional_dijkstra = CHBidirectionalDijkstra::new(&query_graph);

                    let mut ch_bidirectional_dijkstra = CHBidirectionalAStar::new(&query_graph);

                    // let mut ch_bidirectional_dijkstra =
                    //     CHLMAstar::from_landmarks(&query_graph, &weighting, &self.lm, start, end);

                    ch_bidirectional_dijkstra.calc_path(&weighting, start, end, Some(options))
                }
                None => Err(String::from("CH Graph not found")),
            },

            None => {
                let weighting = self.create_weighting(&request.profile);
                let query_graph = QueryGraph::from_graph(&self.graph, &self.graph, &mut snaps[..]);
                let start = snaps[0].closest_node();
                let end = snaps[1].closest_node();
                let mut bdirastar = BidirectionalAStar::new(&query_graph);
                bdirastar.calc_path(&weighting, start, end, Some(options))
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
