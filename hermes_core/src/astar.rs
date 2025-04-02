use crate::astar_heuristic::AStarHeuristic;
use crate::constants::{INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::edge_direction::EdgeDirection;
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::routing_path::{RoutingPath, RoutingPathItem};
use crate::shortest_path_algorithm::{
    ShortestPathAlgorithm, ShortestPathDebugInfo, ShortestPathOptions, ShortestPathResult,
};
use crate::stopwatch::Stopwatch;
use crate::weighting::{Weight, Weighting};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// https://en.wikipedia.org/wiki/A*_search_algorithm

#[derive(Eq, Copy, Clone, Debug)]
struct HeapItem {
    node_id: usize,

    /// g_score is the current cheapest weight from start to node "node_id"
    g_score: Weight,

    /// f_score = g_score + h_score, with h_score being the heuristic value from node_id to the end
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
    fn estimate(&self, graph: &impl Graph, start: usize, end: usize) -> Weight {
        let start_coordinates = graph.node_geometry(start);
        let end_coordinates = graph.node_geometry(end);
        let distance = start_coordinates
            .haversine_distance(end_coordinates)
            .value();

        let speed_kmh = 120.0;
        let speed_ms = speed_kmh / 3.6;

        (distance * 0.7 + (distance / speed_ms).round()) as usize
    }
}

pub struct AStar<H: AStarHeuristic> {
    heap: BinaryHeap<HeapItem>,
    // Use a HashMap instead of a vector. Creating a vector with a capacity of the entire nodes of the planet is not scalable.
    data: HashMap<usize, NodeData>,

    debug_visited_nodes: Option<Vec<usize>>,

    heuristic: H,
}

impl<H: AStarHeuristic> AStar<H> {
    pub fn with_heuristic(_graph: &impl Graph, heuristic: H) -> AStar<H> {
        // TODO: better estimate the capacity to allocate
        let data = HashMap::with_capacity(10000);
        let heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(1024);
        AStar {
            data,
            debug_visited_nodes: None,
            heap,
            heuristic,
        }
    }

    fn init(&mut self, graph: &impl Graph, start: usize, end: usize) {
        let h_score = self.heuristic.estimate(graph, start, end);
        self.heap.push(HeapItem {
            node_id: start,
            g_score: 0,
            f_score: h_score,
        });
        self.update_node_data(start, 0, INVALID_NODE, INVALID_EDGE)
    }

    fn update_node_data(&mut self, node: usize, weight: Weight, parent: usize, edge_id: usize) {
        if let Some(data) = self.data.get_mut(&node) {
            data.weight = weight;
            data.settled = false;
            data.parent = parent;
            data.edge_id = edge_id;
        } else {
            self.data.insert(
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

    fn node_data(&mut self, node: usize) -> &NodeData {
        self.data.entry(node).or_insert_with(NodeData::new)
    }

    fn set_settled(&mut self, node: usize) {
        self.data.get_mut(&node).unwrap().settled = true
    }

    #[inline(always)]
    fn is_settled(&mut self, node: usize) -> bool {
        self.node_data(node).settled
    }

    #[inline(always)]
    fn current_shortest_weight(&mut self, node: usize) -> Weight {
        self.node_data(node).weight
    }

    fn build_path(
        &mut self,
        graph: &impl Graph,
        weighting: &impl Weighting,
        _start: usize,
        end: usize,
    ) -> RoutingPath {
        let mut path: Vec<RoutingPathItem> = Vec::with_capacity(32);

        let mut node = end;

        let mut node_data = self.node_data(node);
        while node_data.parent != INVALID_NODE {
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

            path.push(RoutingPathItem::new(distance, time, geometry));
            node = node_data.parent;
            node_data = self.node_data(node);
        }

        path.reverse();

        RoutingPath::new(path)
    }

    fn add_visited_node(&mut self, node: usize) {
        let debug_visited_nodes = self.debug_visited_nodes.get_or_insert_with(Vec::new);
        debug_visited_nodes.push(node);
    }

    fn debug_info(&self, graph: &impl Graph) -> ShortestPathDebugInfo {
        ShortestPathDebugInfo {
            forward_visited_nodes: self
                .debug_visited_nodes
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|node_id| graph.node_geometry(*node_id))
                .cloned()
                .collect(),
            backward_visited_nodes: vec![],
        }
    }
}

impl<H: AStarHeuristic> ShortestPathAlgorithm for AStar<H> {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &impl Weighting,
        start: usize,
        end: usize,
        options: Option<ShortestPathOptions>,
    ) -> Result<ShortestPathResult, String> {
        let stopwatch = Stopwatch::new("astar/calc_path");
        if start == INVALID_NODE {
            return Err(String::from("AStar: start node is invalid"));
        }

        if end == INVALID_NODE {
            return Err(String::from("AStar: start node is invalid"));
        }

        let include_debug_info: bool = options
            .and_then(|options| options.include_debug_info)
            .unwrap_or(false);

        self.init(graph, start, end);

        let mut iterations = 0;
        let mut nodes_visited = 0;

        while let Some(HeapItem {
            node_id, g_score, ..
        }) = self.heap.pop()
        {
            // Node is already settled, skip
            if self.is_settled(node_id) {
                continue;
            }

            // The weight is bigger than the current shortest weight, skip
            if g_score > self.current_shortest_weight(node_id) {
                continue;
            }

            if g_score > self.current_shortest_weight(end) {
                continue;
            }

            for edge_id in graph.node_edges_iter(node_id) {
                let edge = graph.edge(edge_id);

                let adj_node = edge.adj_node(node_id);

                if self.is_settled(adj_node) {
                    continue;
                }

                let direction = graph.edge_direction(edge_id, node_id);

                let edge_weight = weighting.calc_edge_weight(edge, direction);

                if edge_weight == MAX_WEIGHT {
                    continue;
                }

                let next_weight = g_score + edge_weight;

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight, node_id, edge_id);
                    let h_score = self.heuristic.estimate(graph, adj_node, end);

                    self.heap.push(HeapItem {
                        g_score: next_weight,
                        f_score: next_weight + h_score,
                        node_id: adj_node,
                    });
                }
            }

            if include_debug_info {
                self.add_visited_node(node_id);
            }

            nodes_visited += 1;

            self.set_settled(node_id);
            iterations += 1;
            if node_id == end {
                break;
            }
        }

        println!("AStar iterations: {}", iterations);
        println!("AStar nodes visited: {}", nodes_visited);

        stopwatch.report();

        let path = self.build_path(graph, weighting, start, end);

        Ok(ShortestPathResult {
            path,
            debug: if include_debug_info {
                Some(self.debug_info(graph))
            } else {
                None
            },
        })
    }
}

impl AStar<HaversineHeuristic> {
    pub fn new(graph: &impl Graph) -> AStar<HaversineHeuristic> {
        Self::with_heuristic(graph, HaversineHeuristic)
    }
}
