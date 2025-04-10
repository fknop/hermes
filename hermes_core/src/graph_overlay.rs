use fxhash::{FxHashMap, FxHashSet};

use crate::{
    base_graph::BaseGraph,
    graph::Graph,
    types::{EdgeId, NodeId},
};

pub(crate) struct GraphOverlay<'a, E> {
    base_graph: &'a BaseGraph,

    virtual_nodes: usize,
    virtual_edges: Vec<E>,

    // New edges for new "virtual" nodes
    virtual_adjacency_list: Vec<Vec<EdgeId>>,

    // New edges for existing nodes in the base graph
    virtual_adjacency_list_existing_nodes: FxHashMap<NodeId, Vec<EdgeId>>,

    removed_edges: FxHashSet<EdgeId>,
}

impl<'a, E> GraphOverlay<'a, E> {
    pub fn new(base_graph: &'a BaseGraph) -> Self {
        Self {
            base_graph,
            virtual_nodes: 0,
            virtual_edges: Vec::new(),
            virtual_adjacency_list: Vec::new(),
            virtual_adjacency_list_existing_nodes: FxHashMap::default(),
            removed_edges: FxHashSet::default(),
        }
    }

    pub fn add_edge(&mut self, edge: E, from: NodeId, to: NodeId) {
        let edge_id = self.base_graph.edge_count() + self.virtual_edges.len();
        self.virtual_edges.push(edge);

        if !self.is_virtual_node(from) {
            self.add_virtual_edge_for_existing_node(edge_id, from);
        } else {
            self.virtual_nodes += 1;
        }

        if !self.is_virtual_node(to) {
            self.add_virtual_edge_for_existing_node(edge_id, to);
        } else {
            self.virtual_nodes += 1;
        }

        self.virtual_adjacency_list.push(vec![edge_id]);
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        if self.is_virtual_node(node_id) {
            self.virtual_adjacency_list[self.virtual_node_id(node_id)]
                .clone()
                .iter()
                .for_each(|edge_id| self.remove_edge(*edge_id));
        } else {
            self.base_graph
                .node_edges_iter(node_id)
                .for_each(|edge_id| self.remove_edge(edge_id));
        }
    }

    fn remove_edge(&mut self, edge_id: EdgeId) {
        self.removed_edges.insert(edge_id);
    }

    fn add_virtual_edge_for_existing_node(&mut self, edge_id: usize, node_id: usize) {
        match self.virtual_adjacency_list_existing_nodes.get_mut(&node_id) {
            Some(list) => list.push(edge_id),
            None => {
                self.virtual_adjacency_list_existing_nodes
                    .insert(node_id, vec![edge_id]);
            }
        }
    }

    fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.base_graph.node_count()
    }

    fn is_virtual_edge(&self, edge_id: usize) -> bool {
        edge_id >= self.base_graph.edge_count()
    }

    // Assumes node_id is a virtual node
    fn virtual_node_id(&self, node_id: usize) -> usize {
        node_id - self.base_graph.node_count()
    }

    // Assumes edge_id is a virtual edge
    fn virtual_edge_id(&self, edge_id: usize) -> usize {
        edge_id - self.base_graph.edge_count()
    }

    // Assumes edge_id is a virtual edge
    fn virtual_edge(&self, edge_id: usize) -> &E {
        &self.virtual_edges[self.virtual_edge_id(edge_id)]
    }

    fn node_edges_iter(&self, node_id: usize) -> GraphOverlayIterator<'_> {
        if self.is_virtual_node(node_id) {
            GraphOverlayIterator::new(
                &[],
                &self.virtual_adjacency_list[self.virtual_node_id(node_id)],
                &self.removed_edges,
            )
        } else {
            let virtual_edges = self.virtual_adjacency_list_existing_nodes.get(&node_id);
            let base_edges = self.base_graph.node_edges(node_id);

            match virtual_edges {
                Some(virtual_edges) => {
                    GraphOverlayIterator::new(base_edges, virtual_edges, &self.removed_edges)
                }
                None => GraphOverlayIterator::new(base_edges, &[], &self.removed_edges),
            }
        }
    }
}

pub struct GraphOverlayIterator<'a> {
    base_edges: &'a [usize],
    virtual_edges: &'a [usize],
    removed_edges: &'a FxHashSet<EdgeId>,
    index: usize,
}

impl<'a> GraphOverlayIterator<'a> {
    fn new(
        base_edges: &'a [usize],
        virtual_edges: &'a [usize],
        removed_edges: &'a FxHashSet<EdgeId>,
    ) -> Self {
        GraphOverlayIterator {
            base_edges,
            virtual_edges,
            removed_edges,
            index: 0,
        }
    }
}

impl Iterator for GraphOverlayIterator<'_> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.base_edges.len() {
            let edge = self.base_edges[self.index];
            self.index += 1;

            if self.removed_edges.contains(&edge) {
                continue;
            }

            return Some(edge);
        }

        while self.index - self.base_edges.len() < self.virtual_edges.len() {
            let virtual_index = self.index - self.base_edges.len();
            let edge = self.virtual_edges[virtual_index];
            self.index += 1;

            if self.removed_edges.contains(&edge) {
                continue;
            }

            return Some(edge);
        }

        None
    }
}
