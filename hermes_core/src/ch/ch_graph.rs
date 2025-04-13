use crate::{
    base_graph::BaseGraph,
    constants::{INVALID_EDGE, INVALID_NODE, MAX_DURATION, MAX_WEIGHT},
    distance::{Distance, Meters},
    edge_direction::EdgeDirection,
    geopoint::GeoPoint,
    graph::{DirectedEdgeAccess, GeometryAccess, Graph, UnfoldEdge},
    graph_edge::GraphEdge,
    meters,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

use super::{ch_edge::CHGraphEdge, ch_storage::CHStorage, shortcut::Shortcut};

pub struct CHGraph<'a> {
    storage: &'a CHStorage,
    base_graph: &'a BaseGraph,
}

impl<'a> CHGraph<'a> {
    pub fn new(storage: &'a CHStorage, base_graph: &'a BaseGraph) -> Self {
        Self {
            storage,
            base_graph,
        }
    }
}

impl Graph for CHGraph<'_> {
    type Edge = CHGraphEdge;

    fn edge_count(&self) -> usize {
        self.storage.edge_count()
    }

    fn node_count(&self) -> usize {
        self.storage.nodes_count()
    }

    fn is_virtual_node(&self, _: NodeId) -> bool {
        false
    }

    fn edge(&self, edge_id: EdgeId) -> &Self::Edge {
        self.storage.edge(edge_id)
    }

    fn edge_direction(&self, edge_id: EdgeId, start_node_id: NodeId) -> EdgeDirection {
        let edge = self.edge(edge_id);
        if edge.start_node() == start_node_id {
            EdgeDirection::Forward
        } else {
            EdgeDirection::Backward
        }
    }
}

impl DirectedEdgeAccess for CHGraph<'_> {
    type EdgeIterator<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn node_incoming_edges_iter(&self, node_id: NodeId) -> Self::EdgeIterator<'_> {
        self.storage.incoming_edges(node_id).iter().copied()
    }

    fn node_outgoing_edges_iter(&self, node_id: NodeId) -> Self::EdgeIterator<'_> {
        self.storage.outgoing_edges(node_id).iter().copied()
    }
}

impl UnfoldEdge for CHGraph<'_> {
    // TODO: haven't really looked into it yet, if it's correct or not
    fn unfold_edge(&self, edge: EdgeId, edges: &mut Vec<EdgeId>) {
        match &self.edge(edge) {
            CHGraphEdge::Shortcut(shortcut) => {
                self.unfold_edge(shortcut.incoming_edge, edges);
                self.unfold_edge(shortcut.outgoing_edge, edges);
            }
            CHGraphEdge::Edge(e) => edges.push(e.edge_id),
        }
    }
}

impl GeometryAccess for CHGraph<'_> {
    fn edge_geometry(&self, edge: EdgeId) -> &[GeoPoint] {
        match &self.edge(edge) {
            CHGraphEdge::Edge(base_edge) => self.base_graph.edge_geometry(base_edge.edge_id),
            CHGraphEdge::Shortcut(_) => &[],
        }
    }

    fn node_geometry(&self, node_id: NodeId) -> &GeoPoint {
        self.base_graph.node_geometry(node_id)
    }
}
