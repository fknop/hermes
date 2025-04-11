use crate::{
    distance::{Distance, Meters},
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

use super::shortcut::Shortcut;

pub struct CHEdge {
    from: NodeId,
    to: NodeId,
    distance: Distance<Meters>,
    time: Milliseconds,
    weight: Weight,
}

pub enum CHGraphEdge {
    Shortcut(Shortcut),
    Edge(CHEdge),
}

impl GraphEdge for CHGraphEdge {
    fn start_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut(shortcut) => shortcut.from,
            CHGraphEdge::Edge(edge) => edge.from,
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHGraphEdge::Shortcut(shortcut) => shortcut.to,
            CHGraphEdge::Edge(edge) => edge.to,
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
    adjacency_list_forward: Vec<Vec<EdgeId>>,
    adjacency_list_backward: Vec<Vec<EdgeId>>,
}

impl CHGraph {}
