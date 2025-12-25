use crate::{
    graph::{GeometryAccess, Graph},
    weighting::Weight,
};

pub trait AStarHeuristic {
    fn estimate<G: Graph + GeometryAccess>(&self, graph: &G, start: usize, end: usize) -> Weight;
}
