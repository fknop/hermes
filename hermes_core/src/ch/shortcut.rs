use crate::{
    distance::{Distance, Meters},
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight},
};

pub struct Shortcut {
    pub from: NodeId,
    pub to: NodeId,

    /// Skipped edge incoming to the contracted node
    pub incoming_edge: EdgeId,

    /// Skipped edge outgoing from the contracted node
    pub outgoing_edge: EdgeId,

    pub distance: Distance<Meters>,
    pub time: Milliseconds,
    pub weight: Weight,
}
