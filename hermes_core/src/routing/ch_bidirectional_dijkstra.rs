use fxhash::{FxBuildHasher, FxHashMap};

use crate::ch::ch_edge::CHGraphEdge;
use crate::ch::ch_graph::{CHGraph, NodeRank};
use crate::constants::{DISTANCE_INFLUENCE, INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::edge_direction::EdgeDirection;
use crate::graph::{DirectedEdgeAccess, GeometryAccess, Graph, UndirectedEdgeAccess, UnfoldEdge};

use crate::graph_edge::GraphEdge;
use crate::landmarks::lm_astar_heuristic::LMAstarHeuristic;
use crate::landmarks::lm_data::LMData;
use crate::stopwatch::Stopwatch;
use crate::types::{EdgeId, NodeId};
use crate::weighting::{Weight, Weighting};
use std::cmp::{Ordering, max};
use std::collections::{BinaryHeap, HashMap};

use super::astar_heuristic::AStarHeuristic;
use super::routing_path::{RoutingPath, RoutingPathLeg};
use super::routing_path_builder::build_routing_path;
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
    fn estimate<G: Graph + GeometryAccess>(&self, graph: &G, from: usize, to: usize) -> Weight {
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

pub struct CHBidirectionalAStar<'a, G, H>
where
    G: Graph + DirectedEdgeAccess + UnfoldEdge,
    H: AStarHeuristic,
{
    graph: &'a G,
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

impl<'a, G, H> CHBidirectionalAStar<'a, G, H>
where
    G: Graph + DirectedEdgeAccess + GeometryAccess + UnfoldEdge,
    H: AStarHeuristic,
{
    pub fn with_heuristic(graph: &'a G, heuristic: H) -> Self {
        // Allocate data structures for both search directions
        let forward_data = HashMap::with_capacity_and_hasher(20000, FxBuildHasher::default());
        let forward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(20000);

        let backward_data = HashMap::with_capacity_and_hasher(20000, FxBuildHasher::default());
        let backward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(20000);

        CHBidirectionalAStar {
            graph,
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

    fn init(&mut self, graph: &G, start: usize, end: usize) {
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

    fn node_data_for_direction(&self, dir: SearchDirection) -> &FxHashMap<usize, NodeData> {
        match dir {
            SearchDirection::Forward => &self.forward_data,
            SearchDirection::Backward => &self.backward_data,
        }
    }

    fn node_data_for_direction_mut(
        &mut self,
        dir: SearchDirection,
    ) -> &mut FxHashMap<usize, NodeData> {
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
        let data = self.node_data_for_direction_mut(dir);
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
        let data = self.node_data_for_direction_mut(dir);
        data.entry(node).or_insert_with(NodeData::new)
    }

    fn set_settled(&mut self, dir: SearchDirection, node: usize) {
        self.node_data_for_direction_mut(dir)
            .get_mut(&node)
            .unwrap()
            .settled = true;
    }

    #[inline(always)]
    fn is_settled(&mut self, dir: SearchDirection, node: usize) -> bool {
        self.node_data(dir, node).settled
    }

    #[inline(always)]
    fn current_shortest_weight(&self, dir: SearchDirection, node: NodeId) -> Weight {
        let data = self.node_data_for_direction(dir);
        data.get(&node).map_or(MAX_WEIGHT, |entry| entry.weight)
    }

    fn is_stallable(
        &self,
        weighting: &impl Weighting<G>,
        dir: SearchDirection,
        current: &HeapItem,
    ) -> bool {
        let edges_iter = match dir {
            SearchDirection::Forward => self.graph.node_incoming_edges_iter(current.node_id),
            SearchDirection::Backward => self.graph.node_outgoing_edges_iter(current.node_id),
        };

        for edge_id in edges_iter {
            let edge = self.graph.edge(edge_id);
            let adj_node = edge.adj_node(current.node_id);
            let adj_weight = self.current_shortest_weight(dir, adj_node);
            if adj_weight == MAX_WEIGHT {
                continue;
            }

            let edge_direction = self.graph.edge_direction(
                edge_id,
                match dir {
                    SearchDirection::Forward => adj_node,
                    SearchDirection::Backward => current.node_id,
                },
            );
            let edge_weight = weighting.calc_edge_weight(edge, edge_direction);
            if edge_weight + adj_weight < current.g_score {
                return true;
            }
        }

        false
    }

    fn build_forward_path(&mut self, graph: &G, node: usize) -> Vec<(EdgeId, EdgeDirection)> {
        let mut path: Vec<(EdgeId, EdgeDirection)> = Vec::with_capacity(32);
        let mut current_node = node;

        while let Some(node_data) = self.forward_data.get(&current_node) {
            if node_data.parent == INVALID_NODE {
                break;
            }

            let mut edge_ids = vec![];
            graph.unfold_edge(node_data.edge_id, &mut edge_ids);
            edge_ids.reverse();

            // let mut parent = node_data.parent;
            for edge_id in edge_ids {
                let edge = graph.edge(edge_id);
                let adj_node = edge.adj_node(current_node);
                let direction = graph.edge_direction(edge_id, adj_node);

                path.push((edge_id, direction));
                current_node = adj_node;
            }

            current_node = node_data.parent;
        }

        path.reverse();
        path
    }

    fn build_backward_path(&mut self, graph: &G, node: usize) -> Vec<(EdgeId, EdgeDirection)> {
        let mut path: Vec<(EdgeId, EdgeDirection)> = Vec::with_capacity(32);

        // Start with the first outgoing edge from the meeting node
        let mut current_node = node;

        while let Some(node_data) = self.backward_data.get(&current_node) {
            if node_data.parent == INVALID_NODE {
                break;
            }

            let mut edge_ids = vec![];
            graph.unfold_edge(node_data.edge_id, &mut edge_ids);

            for edge_id in edge_ids {
                let edge = graph.edge(edge_id);
                let adj_node = edge.adj_node(current_node);
                let direction = graph.edge_direction(edge_id, current_node);

                path.push((edge_id, direction));
                current_node = adj_node;
            }

            current_node = node_data.parent;
        }

        path
    }

    fn build_path(
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

        println!("start building path");

        // Get the forward path (start to meeting point)
        let mut forward_path = self.build_forward_path(graph, self.best_meeting_node);

        println!("build_forward_path");

        // Get the backward path (meeting point to end) and append to forward path
        let backward_path = self.build_backward_path(graph, self.best_meeting_node);

        println!("build_backward_path");

        // Combine the two paths
        forward_path.extend(backward_path);

        build_routing_path(graph, weighting, &forward_path)
    }

    fn add_visited_node(&mut self, dir: SearchDirection, node: usize) {
        let debug_visited_nodes = match dir {
            SearchDirection::Forward => &mut self.debug_forward_visited_nodes,
            SearchDirection::Backward => &mut self.debug_backward_visited_nodes,
        }
        .get_or_insert_with(Vec::new);

        debug_visited_nodes.push(node);
    }

    fn debug_info(&self, graph: &G) -> ShortestPathDebugInfo {
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

impl<G, H> CalcPath<G> for CHBidirectionalAStar<'_, G, H>
where
    G: Graph + DirectedEdgeAccess + GeometryAccess + UnfoldEdge + NodeRank,
    H: AStarHeuristic,
{
    fn calc_path(
        &mut self,
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
        self.init(self.graph, start, end);
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
                active_direction = active_direction.opposite();
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

            let current = maybe_item.unwrap();

            let HeapItem {
                node_id,
                g_score,
                f_score,
            } = current;

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
                    .estimate(self.graph, node_id, opposite_direction_target);

            // // Strategy from "Yet another bidirectional algorithm for shortest paths"
            // // Wim Pijls, Henk Post
            // // https://repub.eur.nl/pub/16100/ei2009-10.pdf
            if g_score + opposite_direction_min_f_score - opposite_direction_h
                >= self.best_path_weight
            {
                continue;
            }
            // If this node has already been settled in this direction, skip it
            if self.is_settled(active_direction, node_id) {
                continue;
            }

            // if self.is_stallable(weighting, active_direction, &current) {
            //     continue;
            // }

            // If the weight is bigger than the current shortest weight, skip
            if g_score > self.current_shortest_weight(active_direction, node_id) {
                continue;
            }

            // Check if this node has been visited from the other direction
            let opposite_dir = active_direction.opposite();

            if self.current_shortest_weight(opposite_dir, node_id) != MAX_WEIGHT {
                // We found a meeting point! Calculate the total path weight
                let total_weight = g_score + self.current_shortest_weight(opposite_dir, node_id);
                // If this is better than our best path so far, update it
                if total_weight < self.best_path_weight {
                    self.best_path_weight = total_weight;
                    self.best_meeting_node = node_id;
                }
            }

            let iter = match active_direction {
                SearchDirection::Forward => self.graph.node_outgoing_edges_iter(node_id),
                SearchDirection::Backward => self.graph.node_incoming_edges_iter(node_id),
            };

            // Process all adjacent nodes
            for edge_id in iter {
                let edge = self.graph.edge(edge_id);
                let adj_node = edge.adj_node(node_id);

                if self.is_settled(active_direction, adj_node) {
                    continue;
                }

                let edge_direction = match active_direction {
                    SearchDirection::Forward => self.graph.edge_direction(edge_id, node_id),
                    SearchDirection::Backward => {
                        self.graph.edge_direction(edge_id, node_id).opposite()
                    }
                };

                let edge_weight = weighting.calc_edge_weight(edge, edge_direction);

                if edge_weight == MAX_WEIGHT {
                    continue;
                }

                let next_weight = g_score + edge_weight;

                if next_weight < self.current_shortest_weight(active_direction, adj_node) {
                    self.update_node_data(
                        active_direction,
                        adj_node,
                        next_weight,
                        node_id,
                        edge_id,
                    );

                    // Calculate heuristic
                    let h_score = match active_direction {
                        SearchDirection::Forward => {
                            self.heuristic.estimate(self.graph, adj_node, target)
                        }
                        SearchDirection::Backward => {
                            self.heuristic.estimate(self.graph, adj_node, target)
                        }
                    };

                    self.heap_for_direction(active_direction).push(HeapItem {
                        g_score: next_weight,
                        f_score: next_weight + h_score,
                        node_id: adj_node,
                    });
                }
            }

            self.set_settled(active_direction, node_id);

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

        println!("CHBidirectionalAStar nodes visited: {}", nodes_visited);

        let path = self.build_path(self.graph, weighting, start, end);

        println!("Build path");

        let duration = stopwatch.elapsed();
        stopwatch.report();

        let debug = if include_debug_info {
            Some(self.debug_info(self.graph))
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

impl<'a, G> CHBidirectionalAStar<'a, G, HaversineHeuristic>
where
    G: Graph + DirectedEdgeAccess + GeometryAccess + UnfoldEdge,
{
    pub fn new(graph: &'a G) -> CHBidirectionalAStar<'a, G, HaversineHeuristic> {
        Self::with_heuristic(graph, HaversineHeuristic)
    }
}

pub struct CHDijkstraHeuristic;

impl AStarHeuristic for CHDijkstraHeuristic {
    #[inline(always)]
    fn estimate<G: Graph>(&self, _graph: &G, _start: usize, _end: usize) -> Weight {
        0
    }
}

pub struct CHBidirectionalDijkstra;

/// Dijkstra is simply a variant of AStar with a zero heuristic
impl CHBidirectionalDijkstra {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<G>(graph: &G) -> CHBidirectionalAStar<'_, G, CHDijkstraHeuristic>
    where
        G: Graph + DirectedEdgeAccess + UnfoldEdge + GeometryAccess,
    {
        CHBidirectionalAStar::with_heuristic(graph, CHDijkstraHeuristic)
    }
}

pub struct CHLMAstar;

impl CHLMAstar {
    pub fn from_landmarks<
        'a,
        G: Graph + UnfoldEdge + UndirectedEdgeAccess + DirectedEdgeAccess + GeometryAccess,
    >(
        graph: &'a G,
        weighting: &'a impl Weighting<G>,
        lm: &'a LMData,
        start: usize,
        end: usize,
    ) -> CHBidirectionalAStar<'a, G, LMAstarHeuristic<'a, G, impl Weighting<G>>> {
        let heuristic = LMAstarHeuristic::new(graph, weighting, lm, start, end);
        CHBidirectionalAStar::with_heuristic(graph, heuristic)
    }
}
