use crate::graph::Graph;
use crate::properties::property_map::{BACKWARD_EDGE, EdgeDirection, FORWARD_EDGE};
use crate::routing_path::RoutingPath;
use crate::weighting::Weighting;
use osmpbf::Node;
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
}

const INVALID_NODE: usize = usize::MAX;
const MAX_WEIGHT: usize = usize::MAX;

impl NodeData {
    fn new() -> Self {
        NodeData {
            settled: false,
            weight: MAX_WEIGHT,
            parent: INVALID_NODE,
        }
    }
}

fn update_node(data: &mut Vec<NodeData>, node: usize, weight: usize, parent: usize) {
    data[node].settled = false;
    data[node].weight = weight;
    data[node].parent = parent;
}

pub fn dijkstra(
    graph: &Graph,
    weighting: &impl Weighting,
    from_node: usize,
    to_node: usize,
) -> RoutingPath {
    let mut path = RoutingPath::new();

    let mut data: Vec<_> = (0..graph.get_node_count())
        .map(|_| NodeData::new())
        .collect();
    let mut heap: BinaryHeap<HeapItem> = BinaryHeap::new();

    update_node(&mut data, from_node, 0, INVALID_NODE);
    heap.push(HeapItem {
        node_id: from_node,
        weight: 0,
    });

    while let Some(HeapItem { node_id, weight }) = heap.pop() {
        if data[node_id].settled {
            continue;
        }

        if weight > data[node_id].weight {
            continue;
        }

        for edge_id in graph.get_node_edges(node_id) {
            let edge = graph.get_edge(*edge_id);
            let edge_from_node = edge.get_from_node();
            let edge_to_node = edge.get_to_node();

            let adj_node = if edge_from_node == node_id {
                edge_to_node
            } else {
                edge_from_node
            };
            let direction = if edge_from_node == node_id {
                FORWARD_EDGE
            } else {
                BACKWARD_EDGE
            };

            let edge_weight = weighting.calc_edge_weight(&edge, direction);
            let next_weight = weight + edge_weight;

            if next_weight < data[adj_node].weight {
                update_node(&mut data, adj_node, next_weight, node_id);
                heap.push(HeapItem {
                    weight: next_weight,
                    node_id: adj_node,
                });
            }
        }

        data[node_id].settled = true;
        if node_id == to_node {
            break;
        }
    }

    path
}
