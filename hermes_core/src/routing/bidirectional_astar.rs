use fxhash::{FxBuildHasher, FxHashMap};

use crate::constants::{DISTANCE_INFLUENCE, INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::edge_direction::EdgeDirection;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;

use crate::graph_edge::GraphEdge;
use crate::stopwatch::Stopwatch;
use crate::weighting::{Weight, Weighting};
use std::cmp::{Ordering, max};
use std::collections::{BinaryHeap, HashMap};

use super::astar_heuristic::AStarHeuristic;
use super::routing_path::{RoutingPath, RoutingPathLeg};
use super::search_direction::SearchDirection;
use super::shortest_path_algorithm::{
    CalcPath, CalcPathOptions, CalcPathResult, ShortestPathDebugInfo,
};

/// Bidirectional A* search algorithm
/// Implement strategies from "Yet another bidirectional algorithm for shortest paths"
/// Wim Pijls, Henk Post
/// https://repub.eur.nl/pub/16100/ei2009-10.pdf

#[derive(Eq, Copy, Clone, Debug)]
struct HeapItem {
    node_id: usize,

    /// g_score is the current cheapest weight from start/end to node "node_id"
    g_score: Weight,

    /// f_score = g_score + h_score, with h_score being the heuristic value
    f_score: Weight,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &HeapItem) -> bool {
        self.f_score == other.f_score && self.g_score == other.g_score
    }
}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &HeapItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip weight to make this a min-heap
        other
            .f_score
            .cmp(&self.f_score)
            .then_with(|| other.g_score.cmp(&self.g_score))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

struct NodeData {
    settled: bool,
    weight: Weight,
    parent: usize,
    edge_id: usize, // Edge ID from parent to current node
}

impl NodeData {
    fn new() -> Self {
        NodeData {
            settled: false,
            weight: MAX_WEIGHT,
            parent: INVALID_NODE,
            edge_id: INVALID_EDGE,
        }
    }
}

pub struct HaversineHeuristic;

impl AStarHeuristic for HaversineHeuristic {
    fn estimate(&self, graph: &impl Graph, from: usize, to: usize) -> Weight {
        let start_coordinates = graph.node_geometry(from);
        let end_coordinates = graph.node_geometry(to);
        let distance = start_coordinates
            .haversine_distance(end_coordinates)
            .value();

        let speed_kmh = 120.0;
        let speed_ms = speed_kmh / 3.6;

        (distance * DISTANCE_INFLUENCE + ((distance / speed_ms) * 1000.0).round()) as Weight
    }
}

pub struct BidirectionalAStar<H: AStarHeuristic> {
    // Forward search (from start node)
    forward_heap: BinaryHeap<HeapItem>,
    forward_data: FxHashMap<usize, NodeData>,

    debug_forward_visited_nodes: Option<Vec<usize>>,

    // Backward search (from target node)
    backward_heap: BinaryHeap<HeapItem>,
    backward_data: FxHashMap<usize, NodeData>,

    debug_backward_visited_nodes: Option<Vec<usize>>,

    // Best meeting point and total path weight
    best_meeting_node: usize,
    best_path_weight: Weight,

    heuristic: H,
}

impl<H: AStarHeuristic> BidirectionalAStar<H> {
    pub fn with_heuristic(_graph: &impl Graph, heuristic: H) -> BidirectionalAStar<H> {
        // Allocate data structures for both search directions
        let forward_data = HashMap::with_capacity_and_hasher(20000, FxBuildHasher::default());
        let forward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(20000);

        let backward_data = HashMap::with_capacity_and_hasher(20000, FxBuildHasher::default());
        let backward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(20000);

        BidirectionalAStar {
            forward_data,
            forward_heap,
            backward_data,
            backward_heap,
            best_meeting_node: INVALID_NODE,
            best_path_weight: MAX_WEIGHT,
            heuristic,
            debug_forward_visited_nodes: None,
            debug_backward_visited_nodes: None,
        }
    }

    fn init(&mut self, graph: &impl Graph, start: usize, end: usize) {
        // Initialize forward search from start
        let forward_h_score = self.heuristic.estimate(graph, start, end);
        self.forward_heap.push(HeapItem {
            node_id: start,
            g_score: 0,
            f_score: forward_h_score,
        });
        self.update_node_data(
            SearchDirection::Forward,
            start,
            0,
            INVALID_NODE,
            INVALID_EDGE,
        );

        // Initialize backward search from end
        let backward_h_score = self.heuristic.estimate(graph, end, start);
        self.backward_heap.push(HeapItem {
            node_id: end,
            g_score: 0,
            f_score: backward_h_score,
        });
        self.update_node_data(
            SearchDirection::Backward,
            end,
            0,
            INVALID_NODE,
            INVALID_EDGE,
        );
    }

    fn node_data_for_direction(&mut self, dir: SearchDirection) -> &mut FxHashMap<usize, NodeData> {
        match dir {
            SearchDirection::Forward => &mut self.forward_data,
            SearchDirection::Backward => &mut self.backward_data,
        }
    }

    fn heap_for_direction(&mut self, dir: SearchDirection) -> &mut BinaryHeap<HeapItem> {
        match dir {
            SearchDirection::Forward => &mut self.forward_heap,
            SearchDirection::Backward => &mut self.backward_heap,
        }
    }

    fn update_node_data(
        &mut self,
        dir: SearchDirection,
        node: usize,
        weight: Weight,
        parent: usize,
        edge_id: usize,
    ) {
        let data = self.node_data_for_direction(dir);
        if let Some(node_data) = data.get_mut(&node) {
            node_data.weight = weight;
            node_data.settled = false;
            node_data.parent = parent;
            node_data.edge_id = edge_id;
        } else {
            data.insert(
                node,
                NodeData {
                    weight,
                    settled: false,
                    edge_id,
                    parent,
                },
            );
        }
    }

    fn node_data(&mut self, dir: SearchDirection, node: usize) -> &NodeData {
        let data = self.node_data_for_direction(dir);
        data.entry(node).or_insert_with(NodeData::new)
    }

    fn set_settled(&mut self, dir: SearchDirection, node: usize) {
        self.node_data_for_direction(dir)
            .get_mut(&node)
            .unwrap()
            .settled = true;
    }

    #[inline(always)]
    fn is_settled(&mut self, dir: SearchDirection, node: usize) -> bool {
        self.node_data(dir, node).settled
    }

    #[inline(always)]
    fn current_shortest_weight(&mut self, dir: SearchDirection, node: usize) -> Weight {
        self.node_data(dir, node).weight
    }

    fn process_node<G: Graph>(
        &mut self,
        graph: &G,
        weighting: &dyn Weighting<G>,
        dir: SearchDirection,
        node_id: usize,
        g_score: Weight,
        target: usize,
    ) {
        // If this node has already been settled in this direction, skip it
        if self.is_settled(dir, node_id) {
            return;
        }

        // If the weight is bigger than the current shortest weight, skip
        if g_score > self.current_shortest_weight(dir, node_id) {
            return;
        }

        // If we already found a path and this path is longer, skip
        if g_score > self.best_path_weight {
            return;
        }

        // Check if this node has been visited from the other direction
        let opposite_dir = match dir {
            SearchDirection::Forward => SearchDirection::Backward,
            SearchDirection::Backward => SearchDirection::Forward,
        };

        if self.current_shortest_weight(opposite_dir, node_id) != MAX_WEIGHT {
            // We found a meeting point! Calculate the total path weight
            let total_weight = g_score + self.current_shortest_weight(opposite_dir, node_id);
            // If this is better than our best path so far, update it
            if total_weight < self.best_path_weight {
                self.best_path_weight = total_weight;
                self.best_meeting_node = node_id;
            }
        }

        // Process all adjacent nodes
        for edge_id in graph.node_edges_iter(node_id) {
            let edge = graph.edge(edge_id);
            let adj_node = edge.adj_node(node_id);

            if self.is_settled(dir, adj_node) {
                continue;
            }

            let edge_direction = match dir {
                SearchDirection::Forward => graph.edge_direction(edge_id, node_id),
                SearchDirection::Backward => graph.edge_direction(edge_id, node_id).opposite(),
            };

            let edge_weight = weighting.calc_edge_weight(edge, edge_direction);

            if edge_weight == MAX_WEIGHT {
                continue;
            }

            let next_weight = g_score + edge_weight;

            if next_weight < self.current_shortest_weight(dir, adj_node) {
                self.update_node_data(dir, adj_node, next_weight, node_id, edge_id);

                // Calculate heuristic
                let h_score = match dir {
                    SearchDirection::Forward => self.heuristic.estimate(graph, adj_node, target),
                    SearchDirection::Backward => self.heuristic.estimate(graph, adj_node, target),
                };

                self.heap_for_direction(dir).push(HeapItem {
                    g_score: next_weight,
                    f_score: next_weight + h_score,
                    node_id: adj_node,
                });
            }
        }

        self.set_settled(dir, node_id);
    }

    fn build_forward_path<G: Graph>(
        &mut self,
        graph: &G,
        weighting: &dyn Weighting<G>,
        node: usize,
    ) -> Vec<RoutingPathLeg> {
        let mut path: Vec<RoutingPathLeg> = Vec::with_capacity(32);
        let mut current_node = node;

        while let Some(node_data) = self.forward_data.get(&current_node) {
            if node_data.parent == INVALID_NODE {
                break;
            }

            let edge_id = node_data.edge_id;
            let parent = node_data.parent;
            let direction = graph.edge_direction(edge_id, parent);
            let edge = graph.edge(edge_id);

            let geometry: Vec<GeoPoint> = if direction == EdgeDirection::Forward {
                graph.edge_geometry(edge_id).to_vec()
            } else {
                graph.edge_geometry(edge_id).iter().rev().cloned().collect()
            };

            let distance = edge.distance();
            let time = weighting.calc_edge_ms(edge, direction);

            path.push(RoutingPathLeg::new(distance, time, geometry));
            current_node = parent;
        }

        path.reverse();
        path
    }

    fn build_backward_path<G: Graph>(
        &mut self,
        graph: &G,
        weighting: &dyn Weighting<G>,
        node: usize,
    ) -> Vec<RoutingPathLeg> {
        let mut path: Vec<RoutingPathLeg> = Vec::with_capacity(32);

        // Start with the first outgoing edge from the meeting node
        let mut current_node = node;

        while let Some(node_data) = self.backward_data.get(&current_node) {
            if node_data.parent == INVALID_NODE {
                break;
            }

            let edge_id = node_data.edge_id;
            let parent = node_data.parent;

            // For backward search, we need to reverse the direction
            let direction = graph.edge_direction(edge_id, current_node);

            let edge = graph.edge(edge_id);

            let geometry: Vec<GeoPoint> = if direction == EdgeDirection::Forward {
                graph.edge_geometry(edge_id).to_vec()
            } else {
                graph.edge_geometry(edge_id).iter().rev().cloned().collect()
            };

            let distance = edge.distance();
            let time = weighting.calc_edge_ms(edge, direction);

            path.push(RoutingPathLeg::new(distance, time, geometry));
            current_node = parent;
        }

        path
    }

    fn build_path<G: Graph>(
        &mut self,
        graph: &G,
        weighting: &impl Weighting<G>,
        _start: usize,
        _end: usize,
    ) -> RoutingPath {
        // If no path was found
        if self.best_meeting_node == INVALID_NODE {
            return RoutingPath::new(Vec::new());
        }

        // Get the forward path (start to meeting point)
        let mut forward_path = self.build_forward_path(graph, weighting, self.best_meeting_node);

        // Get the backward path (meeting point to end) and append to forward path
        let backward_path = self.build_backward_path(graph, weighting, self.best_meeting_node);

        // Combine the two paths
        forward_path.extend(backward_path);

        RoutingPath::new(forward_path)
    }

    fn add_visited_node(&mut self, dir: SearchDirection, node: usize) {
        let debug_visited_nodes = match dir {
            SearchDirection::Forward => &mut self.debug_forward_visited_nodes,
            SearchDirection::Backward => &mut self.debug_backward_visited_nodes,
        }
        .get_or_insert_with(Vec::new);

        debug_visited_nodes.push(node);
    }

    fn debug_info(&self, graph: &impl Graph) -> ShortestPathDebugInfo {
        ShortestPathDebugInfo {
            forward_visited_nodes: self
                .debug_forward_visited_nodes
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|node_id| graph.node_geometry(*node_id))
                .cloned()
                .collect(),
            backward_visited_nodes: self
                .debug_backward_visited_nodes
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|node_id| graph.node_geometry(*node_id))
                .cloned()
                .collect(),
        }
    }
}

impl<H: AStarHeuristic> CalcPath for BidirectionalAStar<H> {
    fn calc_path<G: Graph>(
        &mut self,
        graph: &G,
        weighting: &impl Weighting<G>,
        start: usize,
        end: usize,
        options: Option<CalcPathOptions>,
    ) -> Result<CalcPathResult, String> {
        let stopwatch = Stopwatch::new("bidirectional_astar/calc_path");

        if start == INVALID_NODE {
            return Err(String::from("BidirectionalAStar: start node is invalid"));
        }

        if end == INVALID_NODE {
            return Err(String::from("BidirectionalAStar: end node is invalid"));
        }

        let include_debug_info: bool = options
            .and_then(|options| options.include_debug_info)
            .unwrap_or(false);

        // Initialize
        self.init(graph, start, end);
        self.best_meeting_node = INVALID_NODE;
        self.best_path_weight = MAX_WEIGHT;

        let mut nodes_visited = 0;
        let mut active_direction = SearchDirection::Forward;

        // Continue until both heaps are empty or we've found the optimal path
        while !self.forward_heap.is_empty() || !self.backward_heap.is_empty() {
            // If one direction is empty, switch to the other
            if self.forward_heap.is_empty() {
                active_direction = SearchDirection::Backward;
            } else if self.backward_heap.is_empty() {
                active_direction = SearchDirection::Forward;
            }
            // Otherwise alternate directions
            else {
                active_direction = match active_direction {
                    SearchDirection::Forward => SearchDirection::Backward,
                    SearchDirection::Backward => SearchDirection::Forward,
                };
            }

            let (heap, opposite_heap) = match active_direction {
                SearchDirection::Forward => (&mut self.forward_heap, &mut self.backward_heap),
                SearchDirection::Backward => (&mut self.backward_heap, &mut self.forward_heap),
            };

            // Get the current heap for the active direction
            let maybe_item = heap.pop();

            // If there's nothing to process in this direction, skip
            if maybe_item.is_none() {
                continue;
            }

            let HeapItem {
                node_id,
                g_score,
                f_score,
            } = maybe_item.unwrap();

            // If we already found a path and the min f_score is higher
            // than our best path, we can stop the search in this direction
            if g_score > self.best_path_weight || f_score >= self.best_path_weight {
                continue;
            }

            let opposite_direction_min_f_score =
                opposite_heap.peek().map(|item| item.f_score).unwrap_or(0);

            let (target, opposite_direction_target) = match active_direction {
                SearchDirection::Forward => (end, start),
                SearchDirection::Backward => (start, end),
            };

            let opposite_direction_h =
                self.heuristic
                    .estimate(graph, node_id, opposite_direction_target);

            // Strategy from "Yet another bidirectional algorithm for shortest paths"
            // Wim Pijls, Henk Post
            // https://repub.eur.nl/pub/16100/ei2009-10.pdf
            if g_score + opposite_direction_min_f_score - opposite_direction_h
                >= self.best_path_weight
            {
                continue;
            }

            self.process_node(graph, weighting, active_direction, node_id, g_score, target);

            if include_debug_info {
                self.add_visited_node(active_direction, node_id);
            }

            nodes_visited += 1;

            // Check if we can terminate early
            if self.best_meeting_node != INVALID_NODE {
                let min_forward_f = self
                    .forward_heap
                    .peek()
                    .map_or(MAX_WEIGHT, |item| item.f_score);

                let min_backward_f = self
                    .backward_heap
                    .peek()
                    .map_or(MAX_WEIGHT, |item| item.f_score);

                if max(min_forward_f, min_backward_f) >= self.best_path_weight {
                    break;
                }
            }
        }

        println!("BidirectionalAStar nodes visited: {}", nodes_visited);

        let path = self.build_path(graph, weighting, start, end);
        let duration = stopwatch.elapsed();
        stopwatch.report();

        let debug = if include_debug_info {
            Some(self.debug_info(graph))
        } else {
            None
        };

        Ok(CalcPathResult {
            path,
            debug,
            duration,
            nodes_visited,
        })
    }
}

impl BidirectionalAStar<HaversineHeuristic> {
    pub fn new(graph: &impl Graph) -> BidirectionalAStar<HaversineHeuristic> {
        Self::with_heuristic(graph, HaversineHeuristic)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        kilometers,
        routing::shortest_path_algorithm::CalcPath,
        test_graph_utils::test_graph::{RomaniaGraphCity, TestGraph, TestWeighting},
    };

    use super::*;

    #[test]
    fn test_calc_path() {
        let graph = TestGraph::create_romania_graph();

        let mut dijkstra = BidirectionalAStar::new(&graph);
        let weighting = TestWeighting;

        let result = dijkstra.calc_path(
            &graph,
            &weighting,
            RomaniaGraphCity::Oradea.into(),
            RomaniaGraphCity::Bucharest.into(),
            None,
        );

        assert!(result.is_ok());

        let path = result.unwrap().path;
        assert_eq!(path.distance(), kilometers!(429))
    }

    #[test]
    fn test_calc_path_2() {
        let graph = TestGraph::create_romania_graph();

        let mut dijkstra = BidirectionalAStar::new(&graph);
        let weighting = TestWeighting;

        let result = dijkstra.calc_path(
            &graph,
            &weighting,
            RomaniaGraphCity::Iasi.into(),
            RomaniaGraphCity::Timisoara.into(),
            None,
        );

        assert!(result.is_ok());

        let path = result.unwrap().path;
        assert_eq!(path.distance(), kilometers!(855))
    }
}
