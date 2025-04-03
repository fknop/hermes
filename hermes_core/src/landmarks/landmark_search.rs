use crate::constants::{INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::graph::Graph;
use crate::stopwatch::Stopwatch;
use crate::weighting::{Weight, Weighting};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Simple Dijkstra with a start to all nodes

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
    // parent: usize,
    // edge_id: usize,
}

impl NodeData {
    fn new() -> Self {
        NodeData {
            settled: false,
            weight: MAX_WEIGHT,
            // parent: INVALID_NODE,
            // edge_id: INVALID_EDGE,
        }
    }
}

pub struct LandmarkSearch {
    heap: BinaryHeap<HeapItem>,
    // Use a HashMap instead of a vector. Creating a vector with a capacity of the entire nodes of the planet is not scalable.
    data: Vec<NodeData>,

    current_node: usize,
    max_weight: Weight,
}

impl LandmarkSearch {
    pub fn new(graph: &impl Graph, max_weight: Weight) -> Self {
        let mut data = Vec::with_capacity(graph.node_count());

        data.resize_with(graph.node_count(), NodeData::new);

        LandmarkSearch {
            heap: BinaryHeap::with_capacity(1024),
            data,
            current_node: INVALID_NODE,
            max_weight,
        }
    }

    pub fn reset(&mut self) {
        self.heap.clear();
        self.current_node = INVALID_NODE;
        self.data.fill_with(NodeData::new);
    }

    fn init(&mut self, graph: &impl Graph, start: usize) {
        self.current_node = start;
        self.heap.push(HeapItem {
            node_id: start,
            weight: 0,
        });
        self.update_node_data(start, 0 /*, INVALID_NODE, INVALID_EDGE*/)
    }

    fn update_node_data(
        &mut self,
        node: usize,
        weight: Weight, /*, parent: usize, edge_id: usize*/
    ) {
        self.data[node].weight = weight;
        self.data[node].settled = false;
        // self.data[node].parent = parent;
        // self.data[node].edge_id = edge_id;
    }

    #[inline(always)]
    fn node_data(&mut self, node: usize) -> &NodeData {
        &self.data[node]
    }

    #[inline(always)]
    fn set_settled(&mut self, node: usize) {
        self.data[node].settled = true;
    }

    #[inline(always)]
    fn is_settled(&mut self, node: usize) -> bool {
        self.node_data(node).settled
    }

    #[inline(always)]
    fn current_shortest_weight(&mut self, node: usize) -> Weight {
        self.node_data(node).weight
    }

    pub fn find_landmark(
        &mut self,
        graph: &impl Graph,
        weighting: &impl Weighting,
        starts: &[usize],
    ) -> Result<usize, String> {
        let stopwatch = Stopwatch::new("landmark_search/run");

        if self.current_node != INVALID_NODE {
            // Reset if we already did one search before
            self.reset();
        }

        for start in starts.iter() {
            if *start == INVALID_NODE {
                return Err(String::from("AStar: start node is invalid"));
            }
            self.init(graph, *start);
        }

        while let Some(HeapItem {
            node_id, weight, ..
        }) = self.heap.pop()
        {
            self.current_node = node_id;

            if weight > self.max_weight {
                break;
            }

            // Node is already settled, skip
            if self.is_settled(node_id) {
                continue;
            }

            // The weight is bigger than the current shortest weight, skip
            if weight > self.current_shortest_weight(node_id) {
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

                let next_weight = weight + edge_weight;

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight /*, node_id, edge_id*/);

                    self.heap.push(HeapItem {
                        weight: next_weight,
                        node_id: adj_node,
                    });
                }
            }

            self.set_settled(node_id);
        }

        stopwatch.report();

        Ok(self.current_node)
    }
}
