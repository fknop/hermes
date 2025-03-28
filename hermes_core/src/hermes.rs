use crate::a_star::AStar;
use crate::base_graph::BaseGraph;
use crate::dijkstra::Dijkstra;
use crate::geopoint::GeoPoint;
use crate::location_index::LocationIndex;
use crate::query_graph::QueryGraph;
use crate::routing::routing_request::RoutingRequest;
use crate::routing_path::RoutingPath;
use crate::shortest_path_algorithm::ShortestPathAlgorithm;
use crate::stopwatch::Stopwatch;
use crate::weighting::{CarWeighting, Weighting};
use std::collections::HashMap;
use std::path::Path;

pub struct Hermes {
    graph: BaseGraph,
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
        let graph =
            BaseGraph::from_file(graph_file.into_os_string().into_string().unwrap().as_str());
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
        let graph = BaseGraph::from_osm_file(&file_path);
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

    pub fn graph(&self) -> &BaseGraph {
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

        let dijkstra_sw = Stopwatch::new("Dijkstra/calc_path+build_path");

        let mut dijkstra = Dijkstra::new(&query_graph);
        let path = dijkstra.calc_path(&query_graph, weighting, start, end);

        dijkstra_sw.report();

        path
    }

    pub fn closest_edge(&self, profile: String, coordinates: GeoPoint) -> Option<usize> {
        let weighting = self.profiles.get(&profile)?;

        let snap = self
            .index
            .snap(self.graph(), weighting.as_ref(), &coordinates)?;
        Some(snap.edge_id)
    }
}
