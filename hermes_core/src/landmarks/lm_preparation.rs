use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tracing::info;

use crate::{
    base_graph::BaseGraph,
    graph::Graph,
    routing::{
        bidirectional_dijkstra::BidirectionalDijkstra, search_direction::SearchDirection,
        shortest_path_algorithm::ShortestPathAlgorithm,
    },
    weighting::{Weight, Weighting},
};

use super::lm_data::{LMData, Landmark};

pub(crate) struct LMPreparation<'a, W: Weighting<BaseGraph>> {
    graph: &'a BaseGraph,
    weighting: &'a W,
}

impl<'a, W: Weighting<BaseGraph> + Send + Sync> LMPreparation<'a, W> {
    pub fn new(graph: &'a BaseGraph, weighting: &'a W) -> Self {
        Self { graph, weighting }
    }

    pub fn create_landmarks(&self, num_landmarks: usize) -> LMData {
        info!("Start LM preparation");
        let landmarks_ids = self.find_landmarks(num_landmarks);

        info!("Found all {} landmarks", landmarks_ids.len());

        let landmarks: Vec<Landmark> = landmarks_ids
            .par_iter()
            .map(|&node_id| self.create_landmark(node_id))
            .collect();

        info!("Finished LM preparation");

        LMData::new(landmarks)
    }

    fn create_landmark(&self, node_id: usize) -> Landmark {
        let mut weight_from_landmark: Vec<Weight> = Vec::with_capacity(self.graph.node_count());
        let mut weight_to_landmark: Vec<Weight> = Vec::with_capacity(self.graph.node_count());

        let mut landmark_search = BidirectionalDijkstra::with_full_capacity(
            self.graph,
            self.weighting,
            self.graph.node_count(),
        );

        // Compute weights from landmark by using a forward search starting from the landmark
        landmark_search.init_node(node_id, SearchDirection::Forward);
        landmark_search.run(None);

        for node in 0..self.graph.node_count() {
            weight_from_landmark.push(landmark_search.node_weight(node, SearchDirection::Forward));
        }

        landmark_search.reset();

        // Compute weights to landmark by using a backward search starting from the landmark
        landmark_search.init_node(node_id, SearchDirection::Backward);
        landmark_search.run(None);

        for node in 0..self.graph.node_count() {
            weight_to_landmark.push(landmark_search.node_weight(node, SearchDirection::Backward));
        }

        Landmark::new(node_id, weight_from_landmark, weight_to_landmark)
    }

    pub fn find_landmarks(&self, landmarks_count: usize) -> Vec<usize> {
        let mut landmarks: Vec<usize> = Vec::new();

        let mut landmark_search = BidirectionalDijkstra::with_full_capacity(
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

            landmark_search.run(None);

            let result = landmark_search.current_node(SearchDirection::Forward);

            if let Some(landmark) = result {
                landmarks.push(landmark);
            }
        }

        landmarks
    }
}
