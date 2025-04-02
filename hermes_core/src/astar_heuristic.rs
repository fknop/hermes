use crate::{graph::Graph, weighting::Weight};

pub trait AStarHeuristic {
    fn estimate(&self, graph: &impl Graph, start: usize, end: usize) -> Weight;
}
