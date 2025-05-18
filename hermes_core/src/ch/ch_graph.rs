use fxhash::FxHashSet;

use crate::{
    base_graph::BaseGraph,
    edge_direction::EdgeDirection,
    geopoint::GeoPoint,
    graph::{DirectedEdgeAccess, GeometryAccess, Graph, UndirectedEdgeAccess, UnfoldEdge},
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
};

use super::{ch_edge::CHGraphEdge, ch_storage::CHStorage};

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

    pub fn node_incoming_edges(&self, node_id: NodeId) -> &[usize] {
        self.storage.incoming_edges(node_id)
    }

    pub fn node_outgoing_edges(&self, node_id: NodeId) -> &[usize] {
        self.storage.outgoing_edges(node_id)
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
    fn unfold_edge(&self, edge: EdgeId, edges: &mut Vec<EdgeId>) {
        match &self.edge(edge) {
            CHGraphEdge::Shortcut(shortcut) => {
                self.unfold_edge(shortcut.incoming_edge, edges);
                self.unfold_edge(shortcut.outgoing_edge, edges);
            }
            CHGraphEdge::Edge(e) => edges.push(e.id),
        }
    }
}

impl GeometryAccess for CHGraph<'_> {
    fn edge_geometry(&self, edge: EdgeId) -> &[GeoPoint] {
        match &self.edge(edge) {
            CHGraphEdge::Edge(base_edge) => self.base_graph.edge_geometry(base_edge.id),
            CHGraphEdge::Shortcut(_) => {
                panic!("Shortcut don't have geometry, unfold them first")
            }
        }
    }

    fn node_geometry(&self, node_id: NodeId) -> &GeoPoint {
        self.base_graph.node_geometry(node_id)
    }
}

impl UndirectedEdgeAccess for CHGraph<'_> {
    type EdgeIterator<'b>
        = CHUndirectedEdgeIterator<'b>
    where
        Self: 'b;

    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        let incoming_edges = &self.storage.incoming_edges(node_id);
        let outgoing_edges = &self.storage.outgoing_edges(node_id);

        CHUndirectedEdgeIterator::new(&incoming_edges[..], &outgoing_edges[..])
    }
}

pub struct CHUndirectedEdgeIterator<'a> {
    incoming_edges: &'a [EdgeId],
    outgoing_edges: &'a [EdgeId],
    seen: FxHashSet<EdgeId>,
    index: usize,
}

impl<'a> CHUndirectedEdgeIterator<'a> {
    fn new(incoming_edges: &'a [EdgeId], outgoing_edges: &'a [EdgeId]) -> Self {
        CHUndirectedEdgeIterator {
            incoming_edges,
            outgoing_edges,
            index: 0,
            seen: FxHashSet::default(),
        }
    }
}

impl Iterator for CHUndirectedEdgeIterator<'_> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.outgoing_edges.len() {
            let edge = self.outgoing_edges[self.index];
            self.index += 1;

            if self.seen.contains(&edge) {
                continue;
            } else {
                self.seen.insert(edge);
            }

            return Some(edge);
        }

        while self.index - self.outgoing_edges.len() < self.incoming_edges.len() {
            let edge = self.incoming_edges[self.index - self.outgoing_edges.len()];
            self.index += 1;

            if self.seen.contains(&edge) {
                continue;
            } else {
                self.seen.insert(edge);
            }

            return Some(edge);
        }

        None
    }
}

pub trait NodeRank {
    fn node_rank(&self, node_id: NodeId) -> usize;
}

impl NodeRank for CHGraph<'_> {
    fn node_rank(&self, node_id: NodeId) -> usize {
        self.storage.node_rank(node_id)
    }
}
