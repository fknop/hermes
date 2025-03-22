use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::properties::property_map::FORWARD_EDGE;
use crate::routing_path::{RoutingPath, RoutingPathItem};
use crate::weighting::Weighting;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

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
    settled: bool,
    weight: usize,
    parent: usize,
    edge_id: usize, // Edge ID from parent to current node
}

const INVALID_NODE: usize = usize::MAX;
const INVALID_EDGE: usize = usize::MAX;
const MAX_WEIGHT: usize = usize::MAX;

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

pub trait ShortestPathAlgo {
    fn calc_path(
        &mut self,
        graph: &Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> RoutingPath;
}

pub struct Dijkstra {
    heap: BinaryHeap<HeapItem>,
    data: Vec<NodeData>,
}

impl Dijkstra {
    pub fn new(graph: &Graph) -> Self {
        let data: Vec<_> = (0..graph.node_count()).map(|_| NodeData::new()).collect();
        let heap: BinaryHeap<HeapItem> = BinaryHeap::new();
        Dijkstra { heap, data }
    }

    fn init(&mut self, start: usize, end: usize) {
        self.heap.push(HeapItem {
            node_id: start,
            weight: 0,
        });
        self.update_node_data(start, 0, INVALID_NODE, INVALID_EDGE)
    }

    fn update_node_data(&mut self, node: usize, weight: usize, parent: usize, edge_id: usize) {
        self.data[node].settled = false;
        self.data[node].weight = weight;
        self.data[node].parent = parent;
        self.data[node].edge_id = edge_id;
    }

    fn is_settled(&self, node: usize) -> bool {
        self.data[node].settled
    }

    fn current_shortest_weight(&self, node: usize) -> usize {
        self.data[node].weight
    }

    fn build_path(
        &self,
        graph: &Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> RoutingPath {
        let mut path: Vec<RoutingPathItem> = Vec::new();

        let mut node = end;

        while self.data[node].parent != INVALID_NODE {
            let edge_id = self.data[node].edge_id;
            let parent = self.data[node].parent;

            let direction = graph.edge_direction(edge_id, parent);

            let edge = graph.edge(edge_id);

            let geometry: Vec<GeoPoint> = if direction == FORWARD_EDGE {
                graph.edge_geometry(edge_id).iter().cloned().collect()
            } else {
                graph.edge_geometry(edge_id).iter().rev().cloned().collect()
            };

            let distance = edge.distance();
            let time = weighting.calc_edge_ms(edge, direction);

            path.push(RoutingPathItem::new(distance, time, geometry));
            node = self.data[node].parent;
        }

        path.reverse();

        RoutingPath::new(path)
    }
}

impl ShortestPathAlgo for Dijkstra {
    fn calc_path(
        &mut self,
        graph: &Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> RoutingPath {
        self.init(start, end);

        while let Some(HeapItem { node_id, weight }) = self.heap.pop() {
            // Node is already settled, skip
            if self.is_settled(node_id) {
                continue;
            }

            // The weight is bigger than the current shortest weight, skip
            if weight > self.current_shortest_weight(node_id) {
                continue;
            }

            for edge_id in graph.node_edges(node_id) {
                let edge = graph.edge(*edge_id);
                let edge_from_node = edge.start_node();
                let edge_to_node = edge.end_node();

                let adj_node = if edge_from_node == node_id {
                    edge_to_node
                } else {
                    edge_from_node
                };

                let direction = graph.edge_direction(*edge_id, node_id);

                let edge_weight = weighting.calc_edge_weight(&edge, direction);
                let next_weight = if edge_weight == MAX_WEIGHT {
                    MAX_WEIGHT
                } else {
                    weight + edge_weight
                };

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight, node_id, edge_id.clone());
                    self.heap.push(HeapItem {
                        weight: next_weight,
                        node_id: adj_node,
                    });
                }
            }

            self.data[node_id].settled = true;
            if node_id == end {
                break;
            }
        }

        self.build_path(graph, weighting, start, end)
    }
}
