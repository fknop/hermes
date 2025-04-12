use crate::{
    base_graph::BaseGraph,
    constants::{INVALID_EDGE, INVALID_NODE, MAX_DURATION, MAX_WEIGHT},
    distance::{Distance, Meters},
    graph::Graph,
    graph_edge::GraphEdge,
    meters,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

use super::shortcut::Shortcut;

#[derive(Clone)]
pub struct CHBaseEdge {
    pub edge_id: EdgeId,
    pub start: NodeId,
    pub end: NodeId,

    pub distance: Distance<Meters>,
    pub forward_time: Milliseconds,
    pub backward_time: Milliseconds,
    pub forward_weight: Weight,
    pub backward_weight: Weight,
}

#[derive(Clone)]
pub enum CHGraphEdge {
    Shortcut(Shortcut),
    Edge(CHBaseEdge),
}

impl GraphEdge for CHGraphEdge {
    fn start_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut(shortcut) => shortcut.start,
            CHGraphEdge::Edge(edge) => edge.start,
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut(shortcut) => shortcut.end,
            CHGraphEdge::Edge(edge) => edge.end,
        }
    }

    fn distance(&self) -> Distance<Meters> {
        match self {
            CHGraphEdge::Shortcut(shortcut) => shortcut.distance,
            CHGraphEdge::Edge(edge) => edge.distance,
        }
    }

    fn properties(&self) -> &crate::properties::property_map::EdgePropertyMap {
        unimplemented!("This function is not supported for CHGraphEdge")
    }
}

pub struct CHGraph {
    nodes: usize,
    edges: Vec<CHGraphEdge>,
    ranks: Vec<usize>,

    /// For each node, a list the incoming edges into this node
    incoming_edges: Vec<Vec<EdgeId>>,

    /// For each node, a list the outgoing edges from this node
    outgoing_edges: Vec<Vec<EdgeId>>,
}

impl CHGraph {
    pub fn new(base_graph: &BaseGraph) -> Self {
        let edges = vec![
            CHGraphEdge::Edge(CHBaseEdge {
                edge_id: INVALID_EDGE,
                start: INVALID_NODE,
                end: INVALID_NODE,
                forward_weight: MAX_WEIGHT,
                backward_weight: MAX_WEIGHT,
                backward_time: MAX_DURATION,
                forward_time: MAX_DURATION,
                distance: meters!(0)
            });
            base_graph.edge_count()
        ];
        let ranks = vec![0; base_graph.node_count()];
        let incoming_edges = vec![Vec::new(); base_graph.node_count()];
        let outgoing_edges = vec![Vec::new(); base_graph.node_count()];

        Self {
            nodes: base_graph.node_count(),
            edges,
            ranks,
            incoming_edges,
            outgoing_edges,
        }
    }

    pub fn add_edge(&mut self, edge: CHBaseEdge) {
        if edge.forward_weight != MAX_WEIGHT {
            self.outgoing_edges[edge.start].push(edge.edge_id);
            self.incoming_edges[edge.end].push(edge.edge_id);
        }

        if edge.backward_weight != MAX_WEIGHT {
            self.incoming_edges[edge.start].push(edge.edge_id);
            self.outgoing_edges[edge.end].push(edge.edge_id);
        }

        let edge_id = edge.edge_id;
        self.edges[edge_id] = CHGraphEdge::Edge(edge);
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut) {
        let edge_id = self.edges.len();
        self.outgoing_edges[shortcut.start].push(edge_id);
        self.incoming_edges[shortcut.end].push(edge_id);

        self.edges.push(CHGraphEdge::Shortcut(shortcut));
    }
}
