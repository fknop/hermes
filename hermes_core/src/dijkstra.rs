use crate::{
    astar::{AStar, AStarHeuristic},
    graph::Graph,
    weighting::Weight,
};

pub struct DijkstraHeuristic;

impl AStarHeuristic for DijkstraHeuristic {
    #[inline(always)]
    fn estimate(&self, _graph: &impl Graph, _start: usize, _end: usize) -> Weight {
        0
    }
}

pub struct Dijkstra;

/// Dijkstra is simply a variant of AStar with a zero heuristic
impl Dijkstra {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(graph: &impl Graph) -> AStar<DijkstraHeuristic> {
        AStar::with_heuristic(graph, DijkstraHeuristic)
    }
}
