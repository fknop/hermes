use crate::constants::{INVALID_EDGE, INVALID_NODE, MAX_WEIGHT};
use crate::geopoint::GeoPoint;
use crate::graph::Graph;
use crate::properties::property_map::{BACKWARD_EDGE, EdgeDirection, FORWARD_EDGE};
use crate::routing_path::{RoutingPath, RoutingPathItem};
use crate::shortest_path_algorithm::ShortestPathAlgorithm;
use crate::weighting::{Weight, Weighting};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Eq, Copy, Clone, Debug)]
struct HeapItem {
    node_id: usize,
    g_score: Weight, // weight
    f_score: Weight, // g_score + h_score
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &HeapItem) -> bool {
        self.f_score == other.f_score
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

fn get_node_coordinates(graph: &impl Graph, node: usize) -> &GeoPoint {
    let edge = graph.node_edges_iter(node).next().unwrap();
    let edge_direction = graph.edge_direction(edge, node);
    let geometry = graph.edge_geometry(edge);
    match edge_direction {
        FORWARD_EDGE => &geometry[0],
        BACKWARD_EDGE => &geometry[geometry.len() - 1],
    }
}

fn estimate(graph: &impl Graph, start: usize, end: usize) -> Weight {
    let start_coordinates = get_node_coordinates(graph, start);
    let end_coordinates = get_node_coordinates(graph, end);
    let distance = start_coordinates.distance(end_coordinates).value();

    let speed_meters_per_second = 70.0 * (1000.0 / 3600.0);

    (distance / speed_meters_per_second) as usize
}

pub struct AStar {
    heap: BinaryHeap<HeapItem>,
    data: Vec<NodeData>,
}

impl AStar {
    pub fn new(graph: &impl Graph) -> Self {
        let mut data: Vec<NodeData> = Vec::with_capacity(graph.node_count());
        data.resize_with(graph.node_count(), NodeData::new);
        let heap: BinaryHeap<HeapItem> = BinaryHeap::with_capacity(1024);
        AStar { heap, data }
    }

    fn init(&mut self, graph: &impl Graph, start: usize, end: usize) {
        let h_score = estimate(graph, start, end);
        self.heap.push(HeapItem {
            node_id: start,
            g_score: 0,
            f_score: h_score,
        });
        self.update_node_data(start, 0, INVALID_NODE, INVALID_EDGE)
    }

    fn update_node_data(&mut self, node: usize, weight: Weight, parent: usize, edge_id: usize) {
        self.data[node].settled = false;
        self.data[node].weight = weight;
        self.data[node].parent = parent;
        self.data[node].edge_id = edge_id;
    }

    fn is_settled(&self, node: usize) -> bool {
        self.data[node].settled
    }

    fn current_shortest_weight(&self, node: usize) -> Weight {
        self.data[node].weight
    }

    fn build_path(
        &self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        _start: usize,
        end: usize,
    ) -> RoutingPath {
        let mut path: Vec<RoutingPathItem> = Vec::with_capacity(32);

        let mut node = end;

        while self.data[node].parent != INVALID_NODE {
            let edge_id = self.data[node].edge_id;
            let parent = self.data[node].parent;

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
            node = self.data[node].parent;
        }

        path.reverse();

        RoutingPath::new(path)
    }
}

impl ShortestPathAlgorithm for AStar {
    fn calc_path(
        &mut self,
        graph: &impl Graph,
        weighting: &dyn Weighting,
        start: usize,
        end: usize,
    ) -> Result<RoutingPath, String> {
        if start == INVALID_NODE {
            return Err(String::from("Dijkstra: start node is invalid"));
        }

        if end == INVALID_NODE {
            return Err(String::from("Dijkstra: start node is invalid"));
        }

        self.init(graph, start, end);

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

            for edge_id in graph.node_edges_iter(node_id) {
                let edge = graph.edge(edge_id);
                let edge_from_node = edge.start_node();
                let edge_to_node = edge.end_node();

                let adj_node = if edge_from_node == node_id {
                    edge_to_node
                } else {
                    edge_from_node
                };

                let direction = graph.edge_direction(edge_id, node_id);

                let edge_weight = weighting.calc_edge_weight(edge, direction);
                let next_weight = if edge_weight == MAX_WEIGHT {
                    MAX_WEIGHT
                } else {
                    g_score + edge_weight
                };

                if next_weight < self.current_shortest_weight(adj_node) {
                    self.update_node_data(adj_node, next_weight, node_id, edge_id);
                    let h_score = estimate(graph, adj_node, end);
                    self.heap.push(HeapItem {
                        g_score: next_weight,
                        f_score: if next_weight == MAX_WEIGHT {
                            MAX_WEIGHT
                        } else {
                            next_weight + h_score
                        },
                        node_id: adj_node,
                    });
                }
            }

            self.data[node_id].settled = true;
            if node_id == end {
                break;
            }
        }

        Ok(self.build_path(graph, weighting, start, end))
    }
}
