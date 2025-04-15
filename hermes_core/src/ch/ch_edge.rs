use crate::{
    distance::{Distance, Meters},
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

use super::shortcut::Shortcut;

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct CHBaseEdge {
    pub id: EdgeId,
    pub start: NodeId,
    pub end: NodeId,

    pub distance: Distance<Meters>,
    pub forward_time: Milliseconds,
    pub backward_time: Milliseconds,
    pub forward_weight: Weight,
    pub backward_weight: Weight,
}

#[derive(Debug, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
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
