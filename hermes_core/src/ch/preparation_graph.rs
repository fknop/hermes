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

struct Shortcut {
    from: NodeId,
    to: NodeId,

    /// Skipped edge from the "from" node to the contracted node
    from_edge: EdgeId,

    /// Skipped edge from the contracted node to the "to" node
    to_edge: EdgeId,

    distance: Distance<Meters>,
    time: Milliseconds,
    weight: Weight,
}

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
    graph: &'a BaseGraph,

    overlay: GraphOverlay<'a, CHPreparationGraphEdge<'a>>,
}

impl<'a> CHPreparationGraph<'a> {
    fn new(graph: &'a BaseGraph) -> Self {
        CHPreparationGraph {
            graph,
            overlay: GraphOverlay::new(graph),
        }
    }

    fn disconnect_node(&mut self, node_id: NodeId) {
        self.overlay.remove_node(node_id);
    }

    fn add_shortcut(&mut self, shortcut: Shortcut) {
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
        todo!()
    }

    fn node_count(&self) -> usize {
        todo!()
    }

    fn is_virtual_node(&self, node: usize) -> bool {
        todo!()
    }

    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_> {
        todo!()
    }

    fn edge(&self, edge_id: usize) -> &Self::Edge {
        todo!()
    }

    fn edge_geometry(&self, edge_id: usize) -> &[crate::geopoint::GeoPoint] {
        todo!()
    }

    fn node_geometry(&self, node_id: usize) -> &crate::geopoint::GeoPoint {
        todo!()
    }

    fn edge_direction(
        &self,
        edge_id: usize,
        start_node_id: usize,
    ) -> crate::edge_direction::EdgeDirection {
        todo!()
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

impl<'a, W> Weighting<CHPreparationGraph<'a>> for PreparationGraphWeighting<'a, W>
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
