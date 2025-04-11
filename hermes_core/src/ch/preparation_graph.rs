use fxhash::FxHashSet;

use crate::{
    base_graph::{BaseGraph, BaseGraphEdge},
    distance::{Distance, Meters},
    edge_direction::EdgeDirection,
    graph::Graph,
    graph_edge::GraphEdge,
    graph_overlay::{GraphOverlay, GraphOverlayIterator},
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
            CHPreparationGraphEdge::Shortcut(Shortcut { from, .. }) => *from,
            CHPreparationGraphEdge::Edge(base_edge) => base_edge.start_node(),
        }
    }

    fn end_node(&self) -> NodeId {
        match self {
            CHPreparationGraphEdge::Shortcut(Shortcut { to, .. }) => *to,
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
    overlay: GraphOverlay<'a, CHPreparationGraphEdge<'a>>,
}

impl<'a> CHPreparationGraph<'a> {
    pub fn new(graph: &'a BaseGraph) -> Self {
        let edges = graph
            .edges()
            .iter()
            .map(|edge| CHPreparationGraphEdge::Edge(edge))
            .collect();

        CHPreparationGraph {
            edges,
            base_graph: graph,
            overlay: GraphOverlay::new(graph),
        }
    }

    pub fn disconnect_node(&mut self, node_id: NodeId) {
        self.overlay.remove_node(node_id);
    }

    pub fn add_shortcut(&mut self, shortcut: Shortcut) {
        let Shortcut { from, to, .. } = shortcut;
        self.overlay
            .add_edge(CHPreparationGraphEdge::Shortcut(shortcut), from, to);
    }
}

impl<'a> Graph for CHPreparationGraph<'a> {
    type Edge = CHPreparationGraphEdge<'a>;
    type EdgeIterator<'b>
        = GraphOverlayIterator<'b>
    where
        Self: 'b;

    fn edge_count(&self) -> usize {
        self.overlay.edge_count()
    }

    fn node_count(&self) -> usize {
        self.overlay.node_count()
    }

    fn is_virtual_node(&self, node: usize) -> bool {
        self.overlay.is_virtual_node(node)
    }

    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        self.overlay.node_edges_iter(node_id)
    }

    fn edge(&self, edge_id: usize) -> &Self::Edge {
        if self.overlay.is_virtual_edge(edge_id) {
            self.overlay.virtual_edge(edge_id)
        } else {
            &self.edges[edge_id]
        }
    }

    fn edge_geometry(&self, edge_id: usize) -> &[crate::geopoint::GeoPoint] {
        unimplemented!()
    }

    fn node_geometry(&self, node_id: usize) -> &crate::geopoint::GeoPoint {
        unimplemented!()
    }

    fn edge_direction(&self, edge_id: EdgeId, start: NodeId) -> EdgeDirection {
        if self.overlay.is_virtual_edge(edge_id) {
            let edge = self.overlay.virtual_edge(edge_id);

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
            CHPreparationGraphEdge::Shortcut(Shortcut { weight, .. }) => *weight,
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
