use std::hash::BuildHasherDefault;

use fxhash::{FxHashSet, FxHasher};

use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    constants::MAX_WEIGHT,
    distance::{Distance, Meters},
    edge_direction::EdgeDirection,
    graph::Graph,
    graph_edge::GraphEdge,
    properties::property_map::EdgePropertyMap,
    types::{EdgeId, NodeId},
    weighting::{Milliseconds, Weight, Weighting},
};

use super::shortcut::Shortcut;

pub enum CHPreparationGraphEdge<'a> {
    Shortcut(Shortcut),
    Edge(&'a BaseGraphEdge),
}

impl GraphEdge for CHPreparationGraphEdge<'_> {
    fn start_node(&self) -> NodeId {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { start: from, .. }) => *from,
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.start_node(),
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { end: to, .. }) => *to,
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

        for edge in graph.edges() {
            let start_node = edge.start_node();
            let end_node = edge.end_node();

            // From start to end
            let forward_weight = weighting.calc_edge_weight(edge, EdgeDirection::Forward);

            if forward_weight != MAX_WEIGHT {
                outgoing_edges[start_node].push(edge.id());
                incoming_edges[end_node].push(edge.id());
            }

            // From end to start
            let backward_weight = weighting.calc_edge_weight(edge, EdgeDirection::Backward);

            if backward_weight != MAX_WEIGHT {
                incoming_edges[start_node].push(edge.id());
                outgoing_edges[end_node].push(edge.id());
            }
        }

        CHPreparationGraph {
            edges,
            base_graph: graph,
            incoming_edges,
            outgoing_edges,
        }
    }

    fn remove_edge(&mut self, edge_id: EdgeId) {
        let edge = &self.edges[edge_id];
        let start_node = edge.start_node();
        let end_node = edge.end_node();

        self.incoming_edges[start_node].retain(|e| *e != edge_id);
        self.outgoing_edges[start_node].retain(|e| *e != edge_id);

        self.incoming_edges[end_node].retain(|e| *e != edge_id);
        self.outgoing_edges[end_node].retain(|e| *e != edge_id);
    }

    pub fn disconnect_node(&mut self, node_id: NodeId) {
        let mut all_edges = Vec::new();

        if let Some(edges) = self.incoming_edges.get(node_id) {
            all_edges.extend(edges);
        }

        if let Some(edges) = self.outgoing_edges.get(node_id) {
            all_edges.extend(edges);
        }

        for edge_id in all_edges {
            self.remove_edge(edge_id);
        }
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut) {
        let outgoing_edges_for_start = &self.outgoing_edges[shortcut.start];

        // Duplicate shortcut, don't add it
        if outgoing_edges_for_start.contains(&shortcut.end) {
            return;
        }

        // Only accepts directed edges for now
        let edge_id = self.edges.len();

        self.incoming_edges[shortcut.end].push(edge_id);
        self.outgoing_edges[shortcut.start].push(edge_id);

        self.edges.push(CHPreparationGraphEdge::Shortcut(shortcut));
    }

    fn is_shortcut(&self, edge_id: EdgeId) -> bool {
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
    type EdgeIterator<'b>
        = GraphOverlayIterator<'b>
    where
        Self: 'b;

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn node_count(&self) -> usize {
        self.base_graph.node_count()
    }

    fn is_virtual_node(&self, node: usize) -> bool {
        false
    }

    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        let incoming_edges = &self.incoming_edges[node_id];
        let outgoing_edges = &self.outgoing_edges[node_id];

        if incoming_edges.len() + outgoing_edges.len() > 1000 {
            println!(
                "More than 500 edges? {}",
                incoming_edges.len() + outgoing_edges.len()
            )
        }

        GraphOverlayIterator::new(&incoming_edges[..], &outgoing_edges[..])
    }

    fn edge(&self, edge_id: usize) -> &Self::Edge {
        &self.edges[edge_id]
    }

    fn edge_geometry(&self, edge_id: usize) -> &[crate::geopoint::GeoPoint] {
        unimplemented!()
    }

    fn node_geometry(&self, node_id: usize) -> &crate::geopoint::GeoPoint {
        unimplemented!()
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

pub struct PreparationGraphWeighting<'a, W>
where
    W: Weighting<BaseGraph>,
{
    base_graph_weighting: &'a W,
}

impl<'a, W> PreparationGraphWeighting<'a, W>
where
    W: Weighting<BaseGraph>,
{
    pub fn new(base_graph_weighting: &'a W) -> Self {
        Self {
            base_graph_weighting,
        }
    }
}

impl<W> Weighting<CHPreparationGraph<'_>> for PreparationGraphWeighting<'_, W>
where
    W: Weighting<BaseGraph>,
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

pub struct GraphOverlayIterator<'a> {
    incoming_edges: &'a [EdgeId],
    outgoing_edges: &'a [EdgeId],
    seen: FxHashSet<EdgeId>,
    index: usize,
}

impl<'a> GraphOverlayIterator<'a> {
    fn new(incoming_edges: &'a [EdgeId], outgoing_edges: &'a [EdgeId]) -> Self {
        GraphOverlayIterator {
            incoming_edges,
            outgoing_edges,
            index: 0,
            seen: FxHashSet::default(),
        }
    }
}

impl Iterator for GraphOverlayIterator<'_> {
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
