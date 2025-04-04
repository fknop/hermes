use super::{astar::AStar, astar_heuristic::AStarHeuristic};
use crate::{graph::Graph, weighting::Weight};

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

#[cfg(test)]
mod tests {
    use crate::{
        kilometers,
        routing::shortest_path_algorithm::CalcPath,
        test_graph_utils::test_graph::{RomaniaGraphCity, TestGraph, TestWeighting},
    };

    use super::*;

    #[test]
    fn test_calc_path() {
        let graph = TestGraph::create_romania_graph();

        let mut dijkstra = Dijkstra::new(&graph);
        let weighting = TestWeighting;

        let result = dijkstra.calc_path(
            &graph,
            &weighting,
            RomaniaGraphCity::Oradea.into(),
            RomaniaGraphCity::Bucharest.into(),
            None,
        );

        assert!(result.is_ok());

        let path = result.unwrap().path;
        assert_eq!(path.distance(), kilometers!(429))
    }

    #[test]
    fn test_calc_path_2() {
        let graph = TestGraph::create_romania_graph();

        let mut dijkstra = Dijkstra::new(&graph);
        let weighting = TestWeighting;

        let result = dijkstra.calc_path(
            &graph,
            &weighting,
            RomaniaGraphCity::Iasi.into(),
            RomaniaGraphCity::Timisoara.into(),
            None,
        );

        assert!(result.is_ok());

        let path = result.unwrap().path;
        assert_eq!(path.distance(), kilometers!(855))
    }
}
