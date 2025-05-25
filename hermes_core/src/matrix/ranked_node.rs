use std::cmp::Ordering;

use crate::{types::NodeId, weighting::Weight};

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RankedNode {
    pub node_id: NodeId,
    pub rank: usize,
}

impl PartialOrd for RankedNode {
    fn partial_cmp(&self, other: &RankedNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RankedNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Flip weight to make this a min-heap
        other.rank.cmp(&self.rank)
    }
}
