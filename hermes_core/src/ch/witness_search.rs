use std::{cmp::Ordering, collections::BinaryHeap};

use fxhash::FxHashMap;

use crate::{
    constants::{INVALID_NODE, MAX_WEIGHT},
    graph::Graph,
    graph_edge::GraphEdge,
    types::NodeId,
    weighting::{Weight, Weighting},
};

use super::preparation_graph::{CHPreparationGraph, PreparationGraphWeighting};

#[derive(Eq, Copy, Clone, Debug)]
struct HeapItem {
    node_id: usize,
    weight: Weight,
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
    weight: Weight,
}

impl NodeData {
    fn new() -> Self {
        NodeData {
            settled: false,
            weight: MAX_WEIGHT,
        }
    }
}

pub struct WitnessSearch {
    heap: BinaryHeap<HeapItem>,
    data: FxHashMap<NodeId, NodeData>,
    start_node: NodeId,
    avoid_node: NodeId,
    settled_nodes: usize,
}

impl WitnessSearch {
    pub fn new() -> Self {
        WitnessSearch {
            heap: BinaryHeap::default(),
            data: FxHashMap::default(),
            settled_nodes: 0,
            avoid_node: INVALID_NODE,
            start_node: INVALID_NODE,
        }
    }
    pub fn init(&mut self, start_node: NodeId, avoid_node: NodeId) {
        self.heap.clear();
        self.data.clear();

        self.start_node = start_node;
        self.avoid_node = avoid_node;

        self.heap.push(HeapItem {
            node_id: start_node,
            weight: 0,
        });
        self.update_node_data(start_node, 0);
        self.settled_nodes = 0;
    }

    fn update_node_data(&mut self, node: usize, weight: Weight) {
        if let Some(data) = self.data.get_mut(&node) {
            data.weight = weight;
            data.settled = false;
        } else {
            self.data.insert(
                node,
                NodeData {
                    weight,
                    settled: false,
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

    pub fn compute_weight_upperbound<'a>(
        &mut self,
        graph: &CHPreparationGraph<'a>,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        target: NodeId,
        max_weight: Weight,
        max_settled_nodes: usize,
    ) -> Weight {
        if self.start_node == target {
            return 0;
        }

        if self.is_settled(target) {
            return self.current_shortest_weight(target);
        }

        while let Some(HeapItem { weight, node_id }) = self.heap.pop() {
            if self.settled_nodes >= max_settled_nodes {
                break;
            }

            if weight >= max_weight {
                break;
            }

            if self.is_settled(node_id) {
                continue;
            }

            let mut found = false;

            for edge_id in graph.outgoing_edges(node_id) {
                let edge = graph.edge(*edge_id);

                let adj_node = edge.adj_node(node_id);

                if adj_node == self.avoid_node {
                    continue;
                }

                if self.is_settled(adj_node) {
                    continue;
                }

                let direction = graph.edge_direction(*edge_id, node_id);

                let edge_weight = weighting.calc_edge_weight(edge, direction);

                if edge_weight == MAX_WEIGHT {
                    continue;
                }

                let next_weight = weight + edge_weight;

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight);

                    self.heap.push(HeapItem {
                        weight: next_weight,
                        node_id: adj_node,
                    });
                    if adj_node == target && next_weight <= max_weight {
                        found = true;
                        break;
                    }
                }
            }

            self.settled_nodes += 1;
            self.set_settled(node_id);
            if node_id == target || found {
                break;
            }
        }

        self.current_shortest_weight(target)
    }
}
