use crate::{graph::Graph, routing::bidirectional_astar::BidirectionalAStar, weighting::Weighting};

use super::{landmark_heuristic::LandmarkHeuristic, landmarks_data::LandmarksData};

pub struct LandmarksAstar;
impl LandmarksAstar {
    pub fn new<'a>(
        graph: &'a impl Graph,
        weighting: &'a impl Weighting,
        lm: &'a LandmarksData,
        start: usize,
        end: usize,
    ) -> BidirectionalAStar<LandmarkHeuristic<'a, impl Graph, impl Weighting>> {
        let heuristic = LandmarkHeuristic::new(graph, weighting, lm, start, end);
        BidirectionalAStar::with_heuristic(graph, heuristic)
    }
}
