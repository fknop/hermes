use fxhash::FxHashMap;

use crate::{
    geopoint::GeoPoint,
    graph::Graph,
    types::{EdgeId, NodeId},
};

/// A dynamic graph that extends a base graph with virtual nodes and edges
///
/// QueryGraph wraps a base graph and allows for the dynamic addition of virtual nodes
/// and edges during query time. This is particularly useful for routing queries where
/// temporary modifications to the graph are needed without altering the underlying base graph.
///
/// Virtual nodes are typically added when a query point (snap) lies along an existing edge,
/// splitting that edge into two virtual edges connected by the new virtual node.
pub(crate) struct QueryGraphOverlay<'a, G: Graph> {
    query_graph: &'a G,

    virtual_nodes: usize,
    virtual_edges: Vec<G::Edge>,
    virtual_edge_geometry: Vec<Vec<GeoPoint>>,

    // New edges for new "virtual" nodes
    virtual_adjacency_list: Vec<Vec<EdgeId>>,

    // New edges for existing nodes in the base graph
    virtual_adjacency_list_existing_nodes: FxHashMap<NodeId, Vec<EdgeId>>,
}

impl<'a, G: Graph> QueryGraphOverlay<'a, G> {
    pub fn from_graph(graph: &'a G) -> Self {
        Self {
            query_graph: graph,
            virtual_nodes: 0,
            virtual_edges: Vec::new(),
            virtual_edge_geometry: Vec::new(),
            virtual_adjacency_list: Vec::new(),
            virtual_adjacency_list_existing_nodes: FxHashMap::default(),
        }
    }

    pub fn connect_edge(&mut self, edge_id: usize, node_id: NodeId) {
        if self.is_virtual_node(node_id) {
            let virtual_node_id = self.virtual_node_id(node_id);
            if virtual_node_id + 1 > self.virtual_adjacency_list.len() {
                self.virtual_adjacency_list
                    .resize(virtual_node_id + 1, vec![]);
            }

            self.virtual_adjacency_list[virtual_node_id].push(edge_id);
        } else {
            self.add_virtual_edge_for_existing_node(edge_id, node_id);
        }
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

    pub fn add_virtual_edge(&mut self, edge: G::Edge, geometry: Vec<GeoPoint>) {
        self.virtual_edges.push(edge);
        self.virtual_edge_geometry.push(geometry);
    }

    pub fn add_virtual_node(&mut self) -> usize {
        let new_node_id = self.node_count();
        self.virtual_nodes += 1;
        new_node_id
    }

    pub fn virtual_edge_geometry(&self, edge_id: usize) -> &[GeoPoint] {
        &self.virtual_edge_geometry[self.virtual_edge_id(edge_id)]
    }

    pub fn is_virtual_node(&self, node_id: usize) -> bool {
        node_id >= self.query_graph.node_count()
    }

    pub fn is_virtual_edge(&self, edge_id: usize) -> bool {
        edge_id >= self.query_graph.edge_count()
    }

    // Assumes node_id is a virtual node
    pub fn virtual_node_id(&self, node_id: usize) -> usize {
        node_id - self.query_graph.node_count()
    }

    // Assumes edge_id is a virtual edge
    pub fn virtual_edge_id(&self, edge_id: usize) -> usize {
        edge_id - self.query_graph.edge_count()
    }

    // Assumes edge_id is a virtual edge
    pub fn virtual_edge(&self, edge_id: usize) -> &G::Edge {
        &self.virtual_edges[self.virtual_edge_id(edge_id)]
    }

    pub fn edge_count(&self) -> usize {
        self.query_graph.edge_count() + self.virtual_edges.len()
    }

    pub fn node_count(&self) -> usize {
        self.query_graph.node_count() + self.virtual_nodes
    }

    pub fn node_virtual_edges(&self, node_id: usize) -> &[usize] {
        if self.is_virtual_node(node_id) {
            &self.virtual_adjacency_list[self.virtual_node_id(node_id)]
        } else {
            match self.virtual_adjacency_list_existing_nodes.get(&node_id) {
                Some(edges) => edges,
                None => &[],
            }
        }
    }
}
