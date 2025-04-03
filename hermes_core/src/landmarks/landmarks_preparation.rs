use crate::{
    base_graph::BaseGraph,
    graph::Graph,
    routing::{bidirectional_dijkstra::BidirectionalDijkstra, search_direction::SearchDirection},
    weighting::{Weight, Weighting},
};

use super::landmarks_data::{Landmark, LandmarksData};

pub(crate) struct LandmarksPreparation<'a, W: Weighting> {
    graph: &'a BaseGraph,
    weighting: &'a W,
}

impl<'a, W: Weighting> LandmarksPreparation<'a, W> {
    pub fn new(graph: &'a BaseGraph, weighting: &'a W) -> Self {
        Self { graph, weighting }
    }

    pub fn create_landmarks(&self, num_landmarks: usize) -> LandmarksData {
        let landmarks_ids = self.find_landmarks(num_landmarks);
        let mut landmarks = Vec::with_capacity(num_landmarks);

        for node_id in landmarks_ids {
            let landmark = self.create_landmark(node_id);
            landmarks.push(landmark);
        }

        LandmarksData::new(landmarks)
    }

    fn create_landmark(&self, node_id: usize) -> Landmark {
        let mut weight_from_landmark: Vec<Weight> = Vec::with_capacity(self.graph.node_count());
        let mut weight_to_landmark: Vec<Weight> = Vec::with_capacity(self.graph.node_count());

        let mut landmark_search = BidirectionalDijkstra::with_capacity(
            self.graph,
            self.weighting,
            self.graph.node_count(),
        );

        // Compute weights from landmark by using a forward search starting from the landmark
        landmark_search.init_node(node_id, SearchDirection::Forward);
        landmark_search.run();

        for node in 0..self.graph.node_count() {
            weight_from_landmark.push(landmark_search.node_weight(node, SearchDirection::Forward));
        }

        landmark_search.reset();

        // Compute weights to landmark by using a backward search starting from the landmark
        landmark_search.init_node(node_id, SearchDirection::Backward);
        landmark_search.run();

        for node in 0..self.graph.node_count() {
            weight_to_landmark.push(landmark_search.node_weight(node, SearchDirection::Forward));
        }

        Landmark::new(node_id, weight_from_landmark, weight_to_landmark)
    }

    pub fn find_landmarks(&self, landmarks_count: usize) -> Vec<usize> {
        let mut landmarks: Vec<usize> = Vec::new();

        let mut landmark_search = BidirectionalDijkstra::with_capacity(
            self.graph,
            self.weighting,
            self.graph.node_count(),
        );

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
