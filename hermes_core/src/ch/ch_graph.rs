use crate::{
    distance::{Distance, Meters},
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

pub enum CHGraphEdge {
    Shortcut {
        from: NodeId,
        to: NodeId,

        /// Skipped edge from the "from" node to the contracted node
        from_edge: EdgeId,

        /// Skipped edge from the contracted node to the "to" node
        to_edge: EdgeId,

        distance: Distance<Meters>,
        time: Milliseconds,
        weight: Weight,
    },
    Edge {
        from: NodeId,
        to: NodeId,
        distance: Distance<Meters>,
        time: Milliseconds,
        weight: Weight,
    },
}

impl GraphEdge for CHGraphEdge {
    fn start_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut { from, .. } => *from,
            CHGraphEdge::Edge { from, .. } => *from,
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut { to, .. } => *to,
            CHGraphEdge::Edge { to, .. } => *to,
        }
    }

    fn distance(&self) -> Distance<Meters> {
        match self {
            CHGraphEdge::Shortcut { distance, .. } => *distance,
            CHGraphEdge::Edge { distance, .. } => *distance,
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
    adjacency_list_forward: Vec<Vec<EdgeId>>,
    adjacency_list_backward: Vec<Vec<EdgeId>>,
}
