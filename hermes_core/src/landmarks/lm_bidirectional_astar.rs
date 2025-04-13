use crate::{
    graph::{GeometryAccess, Graph, UndirectedEdgeAccess},
    routing::bidirectional_astar::BidirectionalAStar,
    weighting::Weighting,
};

use super::{lm_astar_heuristic::LMAstarHeuristic, lm_data::LMData};

pub struct LMBidirectionalAstar;
impl LMBidirectionalAstar {
    pub fn from_landmarks<'a, G: Graph + UndirectedEdgeAccess + GeometryAccess>(
        graph: &'a G,
        weighting: &'a impl Weighting<G>,
        lm: &'a LMData,
        start: usize,
        end: usize,
    ) -> BidirectionalAStar<LMAstarHeuristic<'a, G, impl Weighting<G>>> {
        let heuristic = LMAstarHeuristic::new(graph, weighting, lm, start, end);
        BidirectionalAStar::with_heuristic(graph, heuristic)
    }
}
