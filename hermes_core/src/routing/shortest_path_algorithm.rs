use crate::{geopoint::GeoPoint, graph::Graph, weighting::Weighting};

use super::routing_path::RoutingPath;

pub struct CalcPathOptions {
    pub include_debug_info: Option<bool>,
}

pub struct ShortestPathDebugInfo {
    pub forward_visited_nodes: Vec<GeoPoint>,
    pub backward_visited_nodes: Vec<GeoPoint>,
}

pub struct CalcPathResult {
    pub path: RoutingPath,
    pub nodes_visited: usize,
    pub debug: Option<ShortestPathDebugInfo>,
}

pub trait CalcPath {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &impl Weighting,
        start: usize,
        end: usize,
        options: Option<CalcPathOptions>,
    ) -> Result<CalcPathResult, String>;
}

pub trait ShortestPathAlgorithm {
    fn run(&mut self, stop_condition: Option<fn(&Self) -> bool>);
    fn finished(&self) -> bool;
}
