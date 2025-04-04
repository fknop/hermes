use std::{cmp, collections::HashMap};

use crate::{
    graph::Graph,
    routing::{
        astar_heuristic::AStarHeuristic, bidirectional_astar::HaversineHeuristic,
        bidirectional_dijkstra::BidirectionalDijkstra, search_direction::SearchDirection,
        shortest_path_algorithm::ShortestPathAlgorithm,
    },
    weighting::{Weight, Weighting},
};

use super::landmarks_data::LandmarksData;

pub struct LandmarkHeuristic<'a, G, W>
where
    G: Graph,
    W: Weighting,
{
    lm: &'a LandmarksData,
    fallback_heuristic: HaversineHeuristic,

    closest_real_nodes: HashMap<usize, (usize, Weight)>,

    graph: &'a G,
    weighting: &'a W,

    start_node: usize,
    end_node: usize,
}

impl<'a, G, W> LandmarkHeuristic<'a, G, W>
where
    G: Graph,
    W: Weighting,
{
    pub fn new(
        graph: &'a G,
        weighting: &'a W,
        lm: &'a LandmarksData,
        start: usize,
        end: usize,
    ) -> Self {
        let mut heuristic = Self {
            lm,
            fallback_heuristic: HaversineHeuristic,
            closest_real_nodes: HashMap::with_capacity(2),
            graph,
            weighting,
            start_node: start,
            end_node: end,
        };

        if graph.is_virtual_node(start) {
            heuristic.insert_closest_real_node(start);
        }

        if graph.is_virtual_node(end) {
            heuristic.insert_closest_real_node(end);
        }

        heuristic
    }

    fn insert_closest_real_node(&mut self, virtual_node_id: usize) {
        let mut algo = BidirectionalDijkstra::with_capacity(self.graph, self.weighting, 2);
        algo.init_node(virtual_node_id, SearchDirection::Forward);
        algo.set_stop_condition(Box::from(
            |current_fwd_node: Option<usize>, _: Option<usize>| {
                if let Some(current_fwd_node) = current_fwd_node {
                    !self.graph.is_virtual_node(current_fwd_node)
                } else {
                    true
                }
            },
        ));

        algo.run();
        let closest_node = algo.current_node(SearchDirection::Forward);

        if let Some(closest_node) = closest_node {
            let weight = algo.node_weight(closest_node, SearchDirection::Forward);
            self.closest_real_nodes
                .insert(virtual_node_id, (closest_node, weight));
        }
    }
}

impl<G, W> AStarHeuristic for LandmarkHeuristic<'_, G, W>
where
    G: Graph,
    W: Weighting,
{
    fn estimate(
        &self,
        graph: &impl Graph,
        maybe_virtual_start: usize,
        maybe_virtual_end: usize,
    ) -> Weight {
        if maybe_virtual_start == maybe_virtual_end {
            return 0;
        }

        let mut start: usize = maybe_virtual_start;
        let mut start_weight_to_real_node = 0;

        let mut end: usize = maybe_virtual_end;
        let mut end_weight_to_real_node = 0;

        let reverse = maybe_virtual_end == self.start_node;

        // Handle virtual nodes
        if graph.is_virtual_node(maybe_virtual_start) {
            let closest = self.closest_real_nodes.get(&maybe_virtual_start);

            if let Some((closest_node, closest_node_weight)) = closest {
                start = *closest_node;
                start_weight_to_real_node = *closest_node_weight;
            } else {
                return self.fallback_heuristic.estimate(
                    graph,
                    maybe_virtual_start,
                    maybe_virtual_end,
                );
            }
        }

        // Handle virtual nodes
        if graph.is_virtual_node(maybe_virtual_end) {
            let closest = self.closest_real_nodes.get(&maybe_virtual_end);

            if let Some((closest_node, closest_node_weight)) = closest {
                end = *closest_node;
                end_weight_to_real_node = *closest_node_weight;
            } else {
                return self.fallback_heuristic.estimate(
                    graph,
                    maybe_virtual_start,
                    maybe_virtual_end,
                );
            }
        }

        let mut lower_bound: Weight = 0;
        for i in 0..self.lm.num_landmarks() {
            let start_to_landmark: i64 =
                (self.lm.weight_to_landmark(i, start) + start_weight_to_real_node) as i64;

            let end_to_landmark: i64 =
                (self.lm.weight_to_landmark(i, end) + end_weight_to_real_node) as i64;

            let landmark_to_start: i64 =
                (self.lm.weight_from_landmark(i, start) + start_weight_to_real_node) as i64;
            let landmark_to_end: i64 =
                (self.lm.weight_from_landmark(i, end) + end_weight_to_real_node) as i64;

            // Triangle inequality: landmark_to_end + landmark_from_start >= weight(start, end)
            // weight > landmark_to_end - landmark_from_start
            // let lm_lower_bound = if reverse {
            //     if end_to_landmark > landmark_to_start {
            //         end_to_landmark - landmark_to_start
            //     } else {
            //         landmark_to_start - end_to_landmark
            //     }
            // } else {
            //     if landmark_to_end > start_to_landmark {
            //         landmark_to_end - start_to_landmark
            //     } else {
            //         start_to_landmark - landmark_to_end
            //     }
            // };
            //

            // TODO: I'm not sure I get this mathematically yet
            // This was inspired by GH
            let mut a = start_to_landmark - end_to_landmark;
            let mut b = landmark_to_end - landmark_to_start;

            if reverse {
                a *= -1;
                b *= -1;
            }

            let lm_lower_bound = cmp::max(0, cmp::max(a, b));

            lower_bound = cmp::max(lower_bound, lm_lower_bound as Weight);
        }

        cmp::max(
            lower_bound,
            self.fallback_heuristic.estimate(graph, start, end),
        )
    }
}
