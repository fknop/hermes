use fxhash::FxHashSet;

use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    constants::MAX_WEIGHT,
    distance::{Distance, Meters},
    edge_direction::EdgeDirection,
    graph::{Graph, UndirectedEdgeAccess},
    graph_edge::GraphEdge,
    properties::property_map::EdgePropertyMap,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight, Weighting},
};

use super::shortcut::{PreparationShortcut, Shortcut};

#[derive(Debug)]
pub enum CHPreparationGraphEdge<'a> {
    Shortcut(Shortcut),
    Edge(&'a BaseGraphEdge),
}

impl GraphEdge for CHPreparationGraphEdge<'_> {
    fn start_node(&self) -> NodeId {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { start, .. }) => *start,
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.start_node(),
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { end, .. }) => *end,
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.end_node(),
        }
    }

    fn distance(&self) -> Distance<Meters> {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { distance, .. }) => *distance,
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.distance(),
        }
    }

    fn properties(&self) -> &EdgePropertyMap {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { .. }) => {
                unimplemented!("edge.properties() for CHPreparationGraphEdge is not implemented")
            }
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.properties(),
        }
    }
}

pub struct CHPreparationGraph<'a> {
    base_graph: &'a BaseGraph,
    edges: Vec<CHPreparationGraphEdge<'a>>,

    // New edges for new "virtual" nodes
    incoming_edges: Vec<Vec<EdgeId>>,
    outgoing_edges: Vec<Vec<EdgeId>>,
    mean_degree: f64,
}

impl<'a> CHPreparationGraph<'a> {
    pub fn new(graph: &'a BaseGraph, weighting: &impl Weighting<BaseGraph>) -> Self {
        let edges = graph
            .edges()
            .iter()
            .map(CHPreparationGraphEdge::Edge)
            .collect();

        let mut incoming_edges = vec![vec![]; graph.node_count()];
        let mut outgoing_edges = vec![vec![]; graph.node_count()];

        let mut relevant_nodes = FxHashSet::default();
        let mut edge_count = 0;

        for edge in graph.edges() {
            let start_node = edge.start_node();
            let end_node = edge.end_node();

            // From start to end
            let forward_weight = weighting.calc_edge_weight(edge, EdgeDirection::Forward);

            if forward_weight != MAX_WEIGHT {
                outgoing_edges[start_node].push(edge.id());
                incoming_edges[end_node].push(edge.id());
                edge_count += 1;
            }

            // From end to start
            let backward_weight = weighting.calc_edge_weight(edge, EdgeDirection::Backward);

            if backward_weight != MAX_WEIGHT {
                incoming_edges[start_node].push(edge.id());
                outgoing_edges[end_node].push(edge.id());
                edge_count += 1;
            }

            if forward_weight != MAX_WEIGHT || backward_weight != MAX_WEIGHT {
                relevant_nodes.insert(start_node);
                relevant_nodes.insert(end_node);
            }
        }

        CHPreparationGraph {
            edges,
            base_graph: graph,
            incoming_edges,
            outgoing_edges,
            mean_degree: edge_count as f64 / relevant_nodes.len() as f64,
        }
    }

    pub fn mean_degree(&self) -> f64 {
        self.mean_degree
    }

    pub fn node_degree(&self, node_id: NodeId) -> usize {
        self.incoming_edges[node_id].len() + self.outgoing_edges[node_id].len()
    }

    fn remove_edge(&mut self, edge_id: EdgeId) {
        let edge = &self.edges[edge_id];
        let start_node = edge.start_node();
        let end_node = edge.end_node();

        self.incoming_edges[start_node].retain(|&e| e != edge_id);
        self.outgoing_edges[start_node].retain(|&e| e != edge_id);

        self.incoming_edges[end_node].retain(|&e| e != edge_id);
        self.outgoing_edges[end_node].retain(|&e| e != edge_id);
    }

    pub fn disconnect_node(&mut self, node_id: NodeId) {
        let mut all_edges = Vec::new();

        if let Some(edges) = self.incoming_edges.get(node_id) {
            all_edges.extend(edges);
        }

        if let Some(edges) = self.outgoing_edges.get(node_id) {
            all_edges.extend(edges);
        }

        let mut neighbors = FxHashSet::default();
        for edge_id in all_edges {
            let adj_node = self.edge(edge_id).adj_node(node_id);
            neighbors.insert(adj_node);
            self.remove_edge(edge_id);
        }

        let degree = neighbors.len();

        // if degree > 0 {
        // Maintain an approximation of a moving average
        self.mean_degree = (self.mean_degree * 2.0 + degree as f64) / 3.0;
        // }
    }

    pub fn add_shortcut(&mut self, shortcut: PreparationShortcut) {
        // Only accepts directed edges for now
        let edge_id = self.edges.len();

        self.outgoing_edges[shortcut.start].push(edge_id);
        self.incoming_edges[shortcut.end].push(edge_id);

        self.edges.push(CHPreparationGraphEdge::Shortcut(Shortcut {
            id: edge_id,
            start: shortcut.start,
            end: shortcut.end,
            weight: shortcut.weight,
            time: shortcut.time,
            distance: shortcut.distance,
            incoming_edge: shortcut.incoming_edge,
            outgoing_edge: shortcut.outgoing_edge,
        }));
    }

    pub fn is_shortcut(&self, edge_id: EdgeId) -> bool {
        edge_id >= self.base_graph.edge_count()
    }

    pub fn incoming_edges(&self, node_id: NodeId) -> &[EdgeId] {
        &self.incoming_edges[node_id]
    }

    pub fn outgoing_edges(&self, node_id: NodeId) -> &[EdgeId] {
        &self.outgoing_edges[node_id]
    }
}

impl<'a> Graph for CHPreparationGraph<'a> {
    type Edge = CHPreparationGraphEdge<'a>;

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn node_count(&self) -> usize {
        self.base_graph.node_count()
    }

    fn is_virtual_node(&self, _: usize) -> bool {
        false
    }

    fn edge(&self, edge_id: usize) -> &Self::Edge {
        &self.edges[edge_id]
    }

    fn edge_direction(&self, edge_id: EdgeId, start: NodeId) -> EdgeDirection {
        if self.is_shortcut(edge_id) {
            let edge = &self.edges[edge_id];

            if edge.start_node() == start {
                return EdgeDirection::Forward;
            } else if edge.end_node() == start {
                return EdgeDirection::Backward;
            }

            panic!(
                "Node {} is neither the start nor the end of edge {}",
                start, edge_id
            )
        } else {
            self.base_graph.edge_direction(edge_id, start)
        }
    }
}

impl UndirectedEdgeAccess for CHPreparationGraph<'_> {
    type EdgeIterator<'b>
        = PreparationGraphEdgeIterator<'b>
    where
        Self: 'b;

    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        let incoming_edges = &self.incoming_edges[node_id];
        let outgoing_edges = &self.outgoing_edges[node_id];

        PreparationGraphEdgeIterator::new(&incoming_edges[..], &outgoing_edges[..])
    }
}

pub struct PreparationGraphWeighting<'a, W>
where
    W: Weighting<BaseGraph> + Send + Sync,
{
    base_graph_weighting: &'a W,
}

impl<'a, W> PreparationGraphWeighting<'a, W>
where
    W: Weighting<BaseGraph> + Send + Sync,
{
    pub fn new(base_graph_weighting: &'a W) -> Self {
        Self {
            base_graph_weighting,
        }
    }
}

impl<W> Weighting<CHPreparationGraph<'_>> for PreparationGraphWeighting<'_, W>
where
    W: Weighting<BaseGraph> + Send + Sync,
{
    fn calc_edge_weight(&self, edge: &CHPreparationGraphEdge, direction: EdgeDirection) -> Weight {
        match edge {
            CHPreparationGraphEdge::Shortcut(Shortcut { weight, .. }) => match direction {
                EdgeDirection::Forward => *weight,
                EdgeDirection::Backward => MAX_WEIGHT,
            },
            CHPreparationGraphEdge::Edge(edge) => {
                self.base_graph_weighting.calc_edge_weight(edge, direction)
            }
        }
    }

    fn calc_edge_ms(
        &self,
        edge: &CHPreparationGraphEdge,
        direction: EdgeDirection,
    ) -> Milliseconds {
        match edge {
            CHPreparationGraphEdge::Shortcut(Shortcut { time, .. }) => *time,
            CHPreparationGraphEdge::Edge(edge) => {
                self.base_graph_weighting.calc_edge_ms(edge, direction)
            }
        }
    }
}

pub struct PreparationGraphEdgeIterator<'a> {
    incoming_edges: &'a [EdgeId],
    outgoing_edges: &'a [EdgeId],
    seen: FxHashSet<EdgeId>,
    index: usize,
}

impl<'a> PreparationGraphEdgeIterator<'a> {
    fn new(incoming_edges: &'a [EdgeId], outgoing_edges: &'a [EdgeId]) -> Self {
        PreparationGraphEdgeIterator {
            incoming_edges,
            outgoing_edges,
            index: 0,
            seen: FxHashSet::default(),
        }
    }
}

impl Iterator for PreparationGraphEdgeIterator<'_> {
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.outgoing_edges.len() {
            let edge = self.outgoing_edges[self.index];
            self.index += 1;

            if self.seen.contains(&edge) {
                continue;
            } else {
                self.seen.insert(edge);
            }

            return Some(edge);
        }

        while self.index - self.outgoing_edges.len() < self.incoming_edges.len() {
            let edge = self.incoming_edges[self.index - self.outgoing_edges.len()];
            self.index += 1;

            if self.seen.contains(&edge) {
                continue;
            } else {
                self.seen.insert(edge);
            }

            return Some(edge);
        }

        None
    }
}
