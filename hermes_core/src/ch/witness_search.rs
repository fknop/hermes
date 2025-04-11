use std::{cmp::Ordering, collections::BinaryHeap};

use fxhash::FxHashMap;

use crate::{
    base_graph::BaseGraph,
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

pub struct WitnessSearch<'a> {
    heap: BinaryHeap<HeapItem>,
    graph: &'a CHPreparationGraph<'a>,
    data: FxHashMap<NodeId, NodeData>,
    start_node: NodeId,
    avoid_node: NodeId,
    settled_nodes: usize,
}

impl<'a> WitnessSearch<'a> {
    pub fn new(graph: &'a CHPreparationGraph) -> Self {
        WitnessSearch {
            graph,
            heap: BinaryHeap::default(),
            data: FxHashMap::default(),
            settled_nodes: 0,
            avoid_node: INVALID_NODE,
            start_node: INVALID_NODE,
        }
    }
}

impl<'a> WitnessSearch<'a> {
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

    pub fn find_max_weight(
        &mut self,
        weighting: &impl Weighting<CHPreparationGraph<'a>>,
        target: NodeId,
        max_weight: Weight,
        max_settled_nodes: usize,
    ) -> Weight {
        while let Some(HeapItem { weight, node_id }) = self.heap.pop() {
            if self.settled_nodes >= max_settled_nodes {
                break;
            }

            if weight > max_weight {
                break;
            }

            if self.is_settled(node_id) {
                continue;
            }

            if weight > self.current_shortest_weight(target) {
                continue;
            }

            for edge_id in self.graph.node_edges_iter(node_id) {
                let edge = self.graph.edge(edge_id);

                let adj_node = edge.adj_node(node_id);

                if adj_node == self.avoid_node {
                    continue;
                }

                if self.is_settled(adj_node) {
                    continue;
                }

                let direction = self.graph.edge_direction(edge_id, node_id);

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
                }
            }

            self.set_settled(node_id);
            if node_id == target {
                break;
            }
        }

        self.current_shortest_weight(target)
    }
}
