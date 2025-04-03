use crate::{
    base_graph::BaseGraph,
    constants::MAX_WEIGHT,
    routing::{bidirectional_dijkstra::BidirectionalDijkstra, search_direction::SearchDirection},
    weighting::Weighting,
};

use super::landmark_search::LandmarkSearch;

pub(crate) struct LandmarksPreparation<'a, W: Weighting> {
    graph: &'a BaseGraph,
    weighting: &'a W,
}

impl<'a, W: Weighting> LandmarksPreparation<'a, W> {
    pub fn new(graph: &'a BaseGraph, weighting: &'a W) -> Self {
        Self { graph, weighting }
    }

    pub fn find_landmarks(&self, landmarks_count: u16) -> Vec<usize> {
        let mut landmarks: Vec<usize> = Vec::new();

        let mut landmark_search = BidirectionalDijkstra::new(self.graph, self.weighting);

        for _ in 0..landmarks_count {
            landmark_search.reset();

            let start_nodes = if landmarks.is_empty() {
                &[0] // Node 0, "random" node
            } else {
                landmarks.as_slice()
            };

            for node in start_nodes {
                landmark_search.init_node(*node, SearchDirection::Forward);
            }

            landmark_search.run();

            let result = landmark_search.current_node(SearchDirection::Forward);

            if let Some(landmark) = result {
                landmarks.push(landmark);
            }
        }

        landmarks
    }
}
