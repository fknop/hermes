
use crate::constants::{INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::properties::property_map::FORWARD_EDGE;
use crate::routing_path::{RoutingPath, RoutingPathItem};
use crate::shortest_path_algorithm::ShortestPathAlgorithm;
use crate::stopwatch::{Stopwatch};
use crate::weighting::{Weight, Weighting};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Eq, Copy, Clone, Debug)]
struct HeapItem {
    node_id: usize,
    weight: usize,
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

struct NodeData {
    weight: Weight,
    settled: bool,
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

pub struct Dijkstra {
    heap: BinaryHeap<HeapItem>,

    // Use a HashMap instead of a vector. Creating a vector with a capacity of the entire nodes of the planet is not scalable.
    data: HashMap<usize, NodeData>,
}

impl Dijkstra {
    pub fn new(graph: &impl Graph) -> Self {
        println!("graph nodes count {}", graph.node_count());
        let data = HashMap::with_capacity(10000);
        let heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(1024);
        Dijkstra { heap, data }
    }

    fn init(&mut self, start: usize, _end: usize) {
        self.heap.push(HeapItem {
            node_id: start,
            weight: 0,
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

    #[inline(always)]
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
        weighting: &dyn Weighting,
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

            let geometry: Vec<GeoPoint> = if direction == FORWARD_EDGE {
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
}

impl ShortestPathAlgorithm for Dijkstra {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> Result<RoutingPath, String> {
        let stopwatch = Stopwatch::new("dijkstra/calc_path");
        if start == INVALID_NODE {
            return Err(String::from("Dijkstra: start node is invalid"));
        }

        if end == INVALID_NODE {
            return Err(String::from("Dijkstra: start node is invalid"));
        }

        self.init(start, end);

        let mut iterations = 0;
        let mut nodes_visited = 0;

        while let Some(HeapItem { node_id, weight }) = self.heap.pop() {
            // Node is already settled, skip
            if self.is_settled(node_id) {
                continue;
            }

            // The weight is bigger than the current shortest weight, skip
            if weight > self.current_shortest_weight(node_id) {
                continue;
            }

            if weight > self.current_shortest_weight(end) {
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

                nodes_visited += 1;

                let next_weight = weight + edge_weight;

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight, node_id, edge_id);
                    self.heap.push(HeapItem {
                        weight: next_weight,
                        node_id: adj_node,
                    });
                }
            }

            self.set_settled(node_id);
            iterations += 1;
            if node_id == end {
                break;
            }
        }

        println!("Dijkstra iterations: {}", iterations);
        println!("Dijkstra nodes visited: {}", nodes_visited);
        stopwatch.report();

        Ok(self.build_path(graph, weighting, start, end))
    }
}
