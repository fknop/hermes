use crate::{base_graph::BaseGraph, constants::MAX_WEIGHT, weighting::Weighting};

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
        let mut landmark_search = LandmarkSearch::new(self.graph, MAX_WEIGHT);

        for _ in 0..landmarks_count {
            let result = landmark_search.find_landmark(
                self.graph,
                self.weighting,
                if landmarks.is_empty() {
                    &[0] // Node 0, "random" node
                } else {
                    landmarks.as_slice()
                },
            );

            if let Ok(landmark) = result {
                landmarks.push(landmark);
            }
        }

        landmarks
    }
}
