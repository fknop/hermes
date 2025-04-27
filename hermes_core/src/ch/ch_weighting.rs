use crate::{
    constants::MAX_WEIGHT,
    edge_direction::EdgeDirection,
    weighting::{Milliseconds, Weight, Weighting},
};

use super::{ch_edge::CHGraphEdge, ch_graph::CHGraph};

pub struct CHWeighting;

impl CHWeighting {
    pub fn new() -> Self {
        CHWeighting
    }
}

impl Weighting<CHGraph<'_>> for CHWeighting {
    fn calc_edge_weight(&self, edge: &CHGraphEdge, direction: EdgeDirection) -> Weight {
        match edge {
            CHGraphEdge::Edge(edge) => match direction {
                EdgeDirection::Forward => edge.forward_weight,
                EdgeDirection::Backward => edge.backward_weight,
            },
            CHGraphEdge::Shortcut(shortcut) => match direction {
                EdgeDirection::Forward => shortcut.weight,
                EdgeDirection::Backward => MAX_WEIGHT,
            },
        }
    }

    fn calc_edge_ms(&self, edge: &CHGraphEdge, direction: EdgeDirection) -> Milliseconds {
        match edge {
            CHGraphEdge::Edge(edge) => match direction {
                EdgeDirection::Forward => edge.forward_time,
                EdgeDirection::Backward => edge.backward_time,
            },
            CHGraphEdge::Shortcut(shortcut) => shortcut.time,
        }
    }
}
