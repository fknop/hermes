use crate::constants::{INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::edge_direction::EdgeDirection;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;

use crate::stopwatch::Stopwatch;
use crate::weighting::{Weight, Weighting};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use super::routing_path::{RoutingPath, RoutingPathLeg};
use super::search_direction::SearchDirection;
use super::shortest_path_algorithm::{ShortestPathAlgorithm, ShortestPathDebugInfo};

#[derive(Eq, Copy, Clone, Debug)]
pub struct HeapItem {
    pub node_id: usize,
    pub weight: Weight,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &HeapItem) -> bool {
        self.weight == other.weight
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
            .weight
            .cmp(&self.weight)
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

pub(crate) struct NodeDataEntry {
    settled: bool,
    weight: Weight,
    parent: usize,
    edge_id: usize, // Edge ID from parent to current node
}

impl NodeDataEntry {
    fn new() -> Self {
        NodeDataEntry {
            settled: false,
            weight: MAX_WEIGHT,
            parent: INVALID_NODE,
            edge_id: INVALID_EDGE,
        }
    }
}

pub(crate) trait NodeData {
    fn clear(&mut self);
    fn get(&self, index: usize) -> Option<&NodeDataEntry>;
    fn get_mut(&mut self, index: usize) -> Option<&mut NodeDataEntry>;
}

pub(crate) struct HashMapNodeData {
    data: HashMap<usize, NodeDataEntry>,
}

impl HashMapNodeData {
    fn with_capacity(capacity_hint: usize) -> Self {
        HashMapNodeData {
            data: HashMap::with_capacity(capacity_hint),
        }
    }
}

impl NodeData for HashMapNodeData {
    fn clear(&mut self) {
        self.data.clear();
    }

    fn get(&self, index: usize) -> Option<&NodeDataEntry> {
        self.data.get(&index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut NodeDataEntry> {
        Some(self.data.entry(index).or_insert_with(NodeDataEntry::new))
    }
}

pub(crate) struct VectorNodeData {
    data: Vec<NodeDataEntry>,
}

impl NodeData for VectorNodeData {
    fn clear(&mut self) {
        self.data.fill_with(NodeDataEntry::new);
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Option<&NodeDataEntry> {
        self.data.get(index)
    }

    #[inline(always)]
    fn get_mut(&mut self, index: usize) -> Option<&mut NodeDataEntry> {
        self.data.get_mut(index)
    }
}

impl VectorNodeData {
    fn with_capacity(capacity_hint: usize) -> Self {
        let mut data = Vec::with_capacity(capacity_hint);
        data.resize_with(capacity_hint, NodeDataEntry::new);
        VectorNodeData { data }
    }
}

type StopCondition<'a> = Box<dyn Fn(Option<usize>, Option<usize>) -> bool + 'a>;

pub(crate) struct BidirectionalDijkstra<'a, G, W, ND>
where
    G: Graph,
    W: Weighting,
    ND: NodeData,
{
    graph: &'a G,
    weighting: &'a W,

    // Forward search (from start node)
    forward_current_node: Option<usize>,
    forward_heap: BinaryHeap<HeapItem>,
    forward_data: ND,

    debug_forward_visited_nodes: Option<Vec<usize>>,

    // Backward search (from target node)
    backward_current_node: Option<usize>,
    backward_heap: BinaryHeap<HeapItem>,
    backward_data: ND,

    debug_backward_visited_nodes: Option<Vec<usize>>,

    // Best meeting point and total path weight
    best_meeting_node: usize,
    best_path_weight: Weight,
    // heuristic: H,
    //
    nodes_visited: usize,

    stop_condition: Option<StopCondition<'a>>,
}

impl<'a, G, W> BidirectionalDijkstra<'a, G, W, VectorNodeData>
where
    G: Graph,
    W: Weighting,
{
    pub fn with_full_capacity(graph: &'a G, weighting: &'a W, capacity_hint: usize) -> Self {
        // Allocate data structures for both search directions
        let forward_data = VectorNodeData::with_capacity(capacity_hint);
        let forward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(capacity_hint / 2);

        let backward_data = VectorNodeData::with_capacity(capacity_hint);
        let backward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(capacity_hint / 2);

        BidirectionalDijkstra {
            graph,
            weighting,
            forward_current_node: None,
            forward_data,
            forward_heap,
            backward_current_node: None,
            backward_data,
            backward_heap,
            best_meeting_node: INVALID_NODE,
            best_path_weight: MAX_WEIGHT,
            // heuristic,
            debug_forward_visited_nodes: None,
            debug_backward_visited_nodes: None,
            nodes_visited: 0,
            stop_condition: None,
        }
    }
}

impl<'a, G, W> BidirectionalDijkstra<'a, G, W, HashMapNodeData>
where
    G: Graph,
    W: Weighting,
{
    pub fn set_stop_condition(&mut self, stop_condition: StopCondition<'a>) {
        self.stop_condition = Some(stop_condition);
    }

    pub fn with_capacity(graph: &'a G, weighting: &'a W, capacity_hint: usize) -> Self {
        // Allocate data structures for both search directions
        let forward_data = HashMapNodeData::with_capacity(capacity_hint);
        let forward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(capacity_hint / 2);

        let backward_data = HashMapNodeData::with_capacity(capacity_hint);
        let backward_heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(capacity_hint / 2);

        BidirectionalDijkstra {
            graph,
            weighting,
            forward_current_node: None,
            forward_data,
            forward_heap,
            backward_current_node: None,
            backward_data,
            backward_heap,
            best_meeting_node: INVALID_NODE,
            best_path_weight: MAX_WEIGHT,
            // heuristic,
            debug_forward_visited_nodes: None,
            debug_backward_visited_nodes: None,
            nodes_visited: 0,
            stop_condition: None,
        }
    }
}

impl<G, W, N> BidirectionalDijkstra<'_, G, W, N>
where
    G: Graph,
    W: Weighting,
    N: NodeData,
{
    pub fn graph(&self) -> &G {
        self.graph
    }

    pub fn current_node(&self, dir: SearchDirection) -> Option<usize> {
        match dir {
            SearchDirection::Forward => self.forward_current_node,
            SearchDirection::Backward => self.backward_current_node,
        }
    }

    pub fn node_weight(&self, node: usize, dir: SearchDirection) -> Weight {
        match dir {
            SearchDirection::Forward => self
                .forward_data
                .get(node)
                .map(|entry| entry.weight)
                .unwrap_or(MAX_WEIGHT),
            SearchDirection::Backward => self
                .backward_data
                .get(node)
                .map(|entry| entry.weight)
                .unwrap_or(MAX_WEIGHT),
        }
    }

    pub fn reset(&mut self) {
        self.forward_data.clear();
        self.forward_heap.clear();
        self.backward_data.clear();
        self.backward_heap.clear();
        self.best_meeting_node = INVALID_NODE;
        self.best_path_weight = MAX_WEIGHT;
        self.debug_forward_visited_nodes = None;
        self.debug_backward_visited_nodes = None;
        self.nodes_visited = 0;
    }

    pub fn init_node(&mut self, node_id: usize, dir: SearchDirection) {
        self.heap_for_direction(dir)
            .push(HeapItem { node_id, weight: 0 });
        self.update_node_data(dir, node_id, 0, INVALID_NODE, INVALID_EDGE);
    }

    fn node_data_for_direction(&mut self, dir: SearchDirection) -> &mut N {
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
        if let Some(entry) = self.node_data_for_direction(dir).get_mut(node) {
            entry.weight = weight;
            entry.parent = parent;
            entry.edge_id = edge_id;
            entry.settled = false
        }
    }

    fn node_data(&mut self, dir: SearchDirection, node: usize) -> &NodeDataEntry {
        let data = self.node_data_for_direction(dir);
        data.get_mut(node).unwrap()
    }

    fn set_settled(&mut self, dir: SearchDirection, node: usize) {
        if let Some(entry) = self.node_data_for_direction(dir).get_mut(node) {
            entry.settled = true
        }
    }

    #[inline(always)]
    fn is_settled(&mut self, dir: SearchDirection, node: usize) -> bool {
        self.node_data(dir, node).settled
    }

    #[inline(always)]
    fn current_shortest_weight(&mut self, dir: SearchDirection, node: usize) -> Weight {
        self.node_data(dir, node).weight
    }

    fn process_node(&mut self, dir: SearchDirection, node_id: usize, weight: Weight) {
        // If this node has already been settled in this direction, skip it
        if self.is_settled(dir, node_id) {
            return;
        }

        // If the weight is bigger than the current shortest weight, skip
        if weight > self.current_shortest_weight(dir, node_id) {
            return;
        }

        // If we already found a path and this path is longer, skip
        if weight > self.best_path_weight {
            return;
        }

        // Check if this node has been visited from the other direction
        let opposite_dir = match dir {
            SearchDirection::Forward => SearchDirection::Backward,
            SearchDirection::Backward => SearchDirection::Forward,
        };

        if self.current_shortest_weight(opposite_dir, node_id) != MAX_WEIGHT {
            // We found a meeting point! Calculate the total path weight
            let total_weight = weight + self.current_shortest_weight(opposite_dir, node_id);
            // If this is better than our best path so far, update it
            if total_weight < self.best_path_weight {
                self.best_path_weight = total_weight;
                self.best_meeting_node = node_id;
            }
        }

        // Process all adjacent nodes
        for edge_id in self.graph.node_edges_iter(node_id) {
            let edge = self.graph.edge(edge_id);
            let adj_node = edge.adj_node(node_id);

            if self.is_settled(dir, adj_node) {
                continue;
            }

            let edge_direction = match dir {
                SearchDirection::Forward => self.graph.edge_direction(edge_id, node_id),
                SearchDirection::Backward => self.graph.edge_direction(edge_id, node_id).opposite(),
            };

            let edge_weight = self.weighting.calc_edge_weight(edge, edge_direction);

            if edge_weight == MAX_WEIGHT {
                continue;
            }

            let next_weight = weight + edge_weight;

            if next_weight < self.current_shortest_weight(dir, adj_node) {
                self.update_node_data(dir, adj_node, next_weight, node_id, edge_id);
                self.heap_for_direction(dir).push(HeapItem {
                    weight: next_weight,
                    node_id: adj_node,
                });
            }
        }

        self.set_settled(dir, node_id);
    }

    fn build_forward_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        node: usize,
    ) -> Vec<RoutingPathLeg> {
        let mut path: Vec<RoutingPathLeg> = Vec::with_capacity(32);
        let mut current_node = node;

        while let Some(node_data) = self.forward_data.get(current_node) {
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

    fn build_backward_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        node: usize,
    ) -> Vec<RoutingPathLeg> {
        let mut path: Vec<RoutingPathLeg> = Vec::with_capacity(32);

        // Start with the first outgoing edge from the meeting node
        let mut current_node = node;

        while let Some(node_data) = self.backward_data.get(current_node) {
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

    fn build_path(
        &mut self,
        graph: &impl Graph,
        weighting: &impl Weighting,
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

impl<G, W, N> ShortestPathAlgorithm for BidirectionalDijkstra<'_, G, W, N>
where
    G: Graph,
    W: Weighting,
    N: NodeData,
{
    fn run(&mut self, stop_condition: Option<fn(&Self) -> bool>) {
        let stopwatch = Stopwatch::new("bidirectional_dijkstra/calc_path");

        let include_debug_info: bool = false; // TODO

        // Initialize
        // self.init(graph, start, end);
        self.best_meeting_node = INVALID_NODE;
        self.best_path_weight = MAX_WEIGHT;

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

            let heap = self.heap_for_direction(active_direction);

            // Get the current heap for the active direction
            let maybe_item = heap.pop();

            // If there's nothing to process in this direction, skip
            if maybe_item.is_none() {
                continue;
            }

            let HeapItem { node_id, weight } = maybe_item.unwrap();

            match active_direction {
                SearchDirection::Forward => {
                    self.forward_current_node = Some(node_id);
                }
                SearchDirection::Backward => self.backward_current_node = Some(node_id),
            }

            // If we already found a path and the min f_score is higher
            // than our best path, we can stop the search in this direction
            if weight > self.best_path_weight {
                continue;
            }

            self.process_node(active_direction, node_id, weight);

            if include_debug_info {
                self.add_visited_node(active_direction, node_id);
            }

            self.nodes_visited += 1;

            // Check if we can terminate early
            if self.finished() {
                break;
            }

            if let Some(ref stop_condition) = stop_condition {
                if stop_condition(self) {
                    break;
                }
            }
        }

        println!(
            "BidirectionalDijkstra nodes visited: {}",
            self.nodes_visited
        );

        stopwatch.report();
    }

    fn finished(&self) -> bool {
        if self.best_meeting_node != INVALID_NODE {
            let min_forward_entry = self.forward_heap.peek();
            let min_backward_entry = self.backward_heap.peek();
            let min_forward_weight = min_forward_entry.map_or(MAX_WEIGHT, |item| item.weight);
            let min_backward_weight = min_backward_entry.map_or(MAX_WEIGHT, |item| item.weight);

            if min_forward_weight + min_backward_weight >= self.best_path_weight {
                return true;
            }
        }

        false
    }
}
