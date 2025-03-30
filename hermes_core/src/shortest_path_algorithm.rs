use crate::{geopoint::GeoPoint, graph::Graph, routing_path::RoutingPath, weighting::Weighting};

pub struct ShortestPathOptions {
    pub include_debug_info: Option<bool>,
}

pub struct ShortestPathDebugInfo {
    pub forward_visited_nodes: Vec<GeoPoint>,
    pub backward_visited_nodes: Vec<GeoPoint>,
}

pub struct ShortestPathResult {
    pub path: RoutingPath,
    pub debug: Option<ShortestPathDebugInfo>,
}

pub trait ShortestPathAlgorithm {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
        options: Option<ShortestPathOptions>,
    ) -> Result<ShortestPathResult, String>;
}
