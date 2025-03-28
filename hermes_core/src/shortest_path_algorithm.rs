use crate::{graph::Graph, routing_path::RoutingPath, weighting::Weighting};

pub trait ShortestPathAlgorithm {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> Result<RoutingPath, String>;
}
