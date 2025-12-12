use std::collections::BinaryHeap;

use fxhash::{FxHashMap, FxHashSet};

use crate::{
    ch::ch_graph::NodeRank,
    constants::MAX_WEIGHT,
    distance::{Distance, Meters},
    graph::{DirectedEdgeAccess, Graph},
    graph_edge::GraphEdge,
    routing::search_direction::SearchDirection,
    stopwatch::Stopwatch,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight, Weighting},
};

use super::{
    matrix::Matrix,
    matrix_algorithm::{MatrixAlgorithm, MatrixAlgorithmResult},
    ranked_node::RankedNode,
};

type Bucket = FxHashMap<NodeId, BucketEntry>;
type Buckets = FxHashMap<NodeId, Bucket>;

type NodeMapping = FxHashMap<NodeId, usize>;

pub struct SBIMatrixAlgorithm<'a, G, W>
where
    G: Graph + DirectedEdgeAccess + NodeRank,
    W: Weighting<G>,
{
    graph: &'a G,
    weighting: &'a W,

    visited_nodes: usize,
    heap: BinaryHeap<RankedNode>,
    forward_buckets: Buckets,
    backward_buckets: Buckets,

    /// Down vertices store the already settled nodes from lower ranks for a given node
    down_vertices: FxHashMap<NodeId, Vec<DownEdge>>,

    /// Up vertices store the nodes from higher ranks for a given node
    up_vertices: FxHashMap<NodeId, Vec<UpEdge>>,

    traversed_nodes: FxHashSet<NodeId>,
}

impl<'a, G, W> SBIMatrixAlgorithm<'a, G, W>
where
    G: Graph + DirectedEdgeAccess + NodeRank,
    W: Weighting<G>,
{
    pub fn new(graph: &'a G, weighting: &'a W) -> Self {
        SBIMatrixAlgorithm {
            graph,
            weighting,
            visited_nodes: 0,
            heap: BinaryHeap::new(),
            forward_buckets: FxHashMap::default(),
            backward_buckets: FxHashMap::default(),

            down_vertices: FxHashMap::default(),
            up_vertices: FxHashMap::default(),
            traversed_nodes: FxHashSet::default(),
        }
    }

    fn accept_edge(&self, edge_id: EdgeId, node: NodeId) -> bool {
        let edge = self.graph.edge(edge_id);
        let adj_node = edge.adj_node(node);
        self.graph.node_rank(node) <= self.graph.node_rank(adj_node)
    }

    fn run_backward_search(&mut self) {
        let direction = SearchDirection::Backward;
        while let Some(current) = self.heap.pop() {
            self.initialize_down_vertices(current.node_id, direction);
            self.initialize_up_vertices(current.node_id, direction);
            self.update_bucket_entries(current.node_id, direction);
            self.retrospective_pruning(current.node_id, direction);
        }
    }

    fn run_forward_search(
        &mut self,
        matrix: &mut Matrix,
        sources_mapping: &NodeMapping,
        targets_mapping: &NodeMapping,
    ) {
        let direction = SearchDirection::Forward;
        while let Some(current) = self.heap.pop() {
            self.initialize_down_vertices(current.node_id, direction);
            self.initialize_up_vertices(current.node_id, direction);
            self.update_bucket_entries(current.node_id, direction);
            self.retrospective_pruning(current.node_id, direction);
            self.find_shortest_paths(current.node_id, matrix, sources_mapping, targets_mapping);
        }
    }

    fn initialize_backward_search(&mut self, targets: &[NodeId]) {
        for &target in targets {
            self.heap.push(RankedNode {
                node_id: target,
                rank: self.graph.node_rank(target),
            });

            let target_buckets = self.backward_buckets.entry(target).or_default();
            target_buckets.insert(
                target,
                BucketEntry {
                    weight: 0,
                    time: 0,
                    distance: Distance::default(),
                },
            );
        }
    }

    fn initialize_forward_search(&mut self, sources: &[NodeId]) {
        self.heap.clear();
        self.traversed_nodes.clear();
        self.up_vertices.clear();
        self.down_vertices.clear();

        for &source in sources {
            self.heap.push(RankedNode {
                node_id: source,
                rank: self.graph.node_rank(source),
            });

            let source_buckets = self.forward_buckets.entry(source).or_default();
            source_buckets.insert(
                source,
                BucketEntry {
                    weight: 0,
                    time: 0,
                    distance: Distance::default(),
                },
            );
        }
    }

    fn initialize_down_vertices(&mut self, node: NodeId, search_direction: SearchDirection) {
        let iter = match search_direction {
            SearchDirection::Forward => self.graph.node_outgoing_edges_iter(node),
            SearchDirection::Backward => self.graph.node_incoming_edges_iter(node),
        };

        for edge_id in iter {
            if !self.accept_edge(edge_id, node) {
                continue;
            }

            let edge = self.graph.edge(edge_id);
            let adj_node = edge.adj_node(node);

            let edge_direction = match search_direction {
                SearchDirection::Forward => self.graph.edge_direction(edge_id, node),
                SearchDirection::Backward => self.graph.edge_direction(edge_id, adj_node),
            };
            let weight = self.weighting.calc_edge_ms(edge, edge_direction);

            if weight == MAX_WEIGHT {
                continue;
            }

            // If the node has not been visited yet, we add it to the heap
            if !self.traversed_nodes.contains(&adj_node) {
                self.visited_nodes += 1;
                self.heap.push(RankedNode {
                    node_id: adj_node,
                    rank: self.graph.node_rank(adj_node),
                });

                self.traversed_nodes.insert(adj_node);
            }

            self.down_vertices
                .entry(adj_node)
                .or_default()
                .push(DownEdge {
                    node_id: node,
                    weight,
                    time: self.weighting.calc_edge_ms(edge, edge_direction),
                    distance: edge.distance(),
                });
        }
    }

    fn initialize_up_vertices(&mut self, node: NodeId, search_direction: SearchDirection) {
        let iter = match search_direction {
            SearchDirection::Forward => self.graph.node_incoming_edges_iter(node),
            SearchDirection::Backward => self.graph.node_outgoing_edges_iter(node),
        };

        for edge_id in iter {
            if !self.accept_edge(edge_id, node) {
                continue;
            }

            let edge = self.graph.edge(edge_id);
            let adj_node = edge.adj_node(node);
            let weight = self.weighting.calc_edge_ms(
                edge,
                match search_direction {
                    SearchDirection::Forward => self.graph.edge_direction(edge_id, adj_node),
                    SearchDirection::Backward => self.graph.edge_direction(edge_id, node),
                },
            );

            if weight == MAX_WEIGHT {
                continue;
            }

            self.up_vertices.entry(adj_node).or_default().push(UpEdge {
                node_id: node,
                weight,
            });
        }
    }

    fn update_bucket_entries(&mut self, node: NodeId, search_direction: SearchDirection) {
        if let Some(down_vertices) = self.down_vertices.get(&node) {
            let mut bucket = self
                .node_bucket(node, search_direction)
                .cloned()
                .unwrap_or_default();

            for down_edge in down_vertices {
                if let Some(adj_bucket) = self.node_bucket(down_edge.node_id, search_direction) {
                    for (&source_or_target, entry) in adj_bucket {
                        let current_weight = bucket
                            .get(&source_or_target)
                            .map(|entry| entry.weight)
                            .unwrap_or(MAX_WEIGHT);

                        let new_weight = down_edge.weight + entry.weight;

                        if current_weight > new_weight {
                            bucket.insert(
                                source_or_target,
                                BucketEntry {
                                    weight: new_weight,
                                    time: entry.time + down_edge.time,
                                    distance: entry.distance + down_edge.distance,
                                },
                            );
                        }
                    }
                }
            }

            match search_direction {
                SearchDirection::Forward => self.forward_buckets.insert(node, bucket),
                SearchDirection::Backward => self.backward_buckets.insert(node, bucket),
            };
        }
    }

    fn retrospective_pruning(&mut self, node: NodeId, search_direction: SearchDirection) {
        if let Some(up_vertices) = self.up_vertices.get(&node) {
            let node_bucket = self.node_bucket(node, search_direction);

            let mut nodes_to_prune = vec![];
            for up_edge in up_vertices {
                if let Some(bucket) = self.node_bucket(up_edge.node_id, search_direction) {
                    for (&source_or_target, entry) in bucket {
                        if let Some(current_weight) = node_bucket
                            .and_then(|bucket| bucket.get(&source_or_target))
                            .map(|entry| entry.weight)
                            && entry.weight > up_edge.weight + current_weight {
                                nodes_to_prune.push((up_edge.node_id, source_or_target));
                            }
                    }
                }
            }

            for (adj_node, source_or_target) in nodes_to_prune {
                if let Some(bucket) = self.node_bucket_mut(adj_node, search_direction) {
                    bucket.remove(&source_or_target);
                }
            }
        }
    }

    fn find_shortest_paths(
        &mut self,
        node: NodeId,
        matrix: &mut Matrix,
        sources_mapping: &NodeMapping,
        targets_mapping: &NodeMapping,
    ) {
        let node_forward_bucket = self.forward_buckets.get(&node).unwrap();
        if let Some(node_backward_bucket) = self.backward_buckets.get(&node) {
            for (&target, backward_entry) in node_backward_bucket {
                for (&source, forward_entry) in node_forward_bucket {
                    let current_weight =
                        matrix.weight(sources_mapping[&source], targets_mapping[&target]);
                    let new_weight = forward_entry.weight + backward_entry.weight;
                    if current_weight > new_weight {
                        matrix.update_entry(
                            sources_mapping[&source],
                            targets_mapping[&target],
                            new_weight,
                            forward_entry.distance + backward_entry.distance,
                            forward_entry.time + backward_entry.time,
                        );
                    }
                }
            }
        }
    }

    fn node_bucket(&self, node: NodeId, search_direction: SearchDirection) -> Option<&Bucket> {
        match search_direction {
            SearchDirection::Forward => self.forward_buckets.get(&node),
            SearchDirection::Backward => self.backward_buckets.get(&node),
        }
    }

    fn node_bucket_mut(
        &mut self,
        node: NodeId,
        search_direction: SearchDirection,
    ) -> Option<&mut Bucket> {
        match search_direction {
            SearchDirection::Forward => self.forward_buckets.get_mut(&node),
            SearchDirection::Backward => self.backward_buckets.get_mut(&node),
        }
    }
}

impl<G, W> MatrixAlgorithm for SBIMatrixAlgorithm<'_, G, W>
where
    G: Graph + DirectedEdgeAccess + NodeRank,
    W: Weighting<G>,
{
    fn calc_matrix(&mut self, sources: &[NodeId], targets: &[NodeId]) -> MatrixAlgorithmResult {
        let mut stopwatch = Stopwatch::new(String::from("calc_matrix"));
        stopwatch.start();

        self.visited_nodes = 0;
        let mut matrix = Matrix::new(sources.len(), targets.len());

        let sources_mapping: NodeMapping = sources
            .iter()
            .enumerate()
            .map(|(index, &source)| (source, index))
            .collect();

        let targets_mapping: NodeMapping = targets
            .iter()
            .enumerate()
            .map(|(index, &target)| (target, index))
            .collect();

        self.initialize_backward_search(targets);
        self.run_backward_search();

        self.initialize_forward_search(sources);
        self.run_forward_search(&mut matrix, &sources_mapping, &targets_mapping);

        stopwatch.stop();
        MatrixAlgorithmResult {
            matrix,
            visited_nodes: self.visited_nodes,
            duration: stopwatch.elapsed(),
        }
    }
}

struct DownEdge {
    pub node_id: NodeId,
    pub weight: Weight,
    pub time: Milliseconds,
    pub distance: Distance<Meters>,
}

struct UpEdge {
    node_id: NodeId,
    pub weight: Weight,
}

#[derive(Clone)]
struct BucketEntry {
    pub weight: Weight,
    pub time: Milliseconds,
    pub distance: Distance<Meters>,
}
