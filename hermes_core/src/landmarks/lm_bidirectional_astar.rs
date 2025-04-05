use crate::{graph::Graph, routing::bidirectional_astar::BidirectionalAStar, weighting::Weighting};

use super::{lm_astar_heuristic::LMAstarHeuristic, lm_data::LMData};

pub struct LMBidirectionalAstar;
impl LMBidirectionalAstar {
    pub fn from_landmarks<'a>(
        graph: &'a impl Graph,
        weighting: &'a impl Weighting,
        lm: &'a LMData,
        start: usize,
        end: usize,
    ) -> BidirectionalAStar<LMAstarHeuristic<'a, impl Graph, impl Weighting>> {
        let heuristic = LMAstarHeuristic::new(graph, weighting, lm, start, end);
        BidirectionalAStar::with_heuristic(graph, heuristic)
    }
}
