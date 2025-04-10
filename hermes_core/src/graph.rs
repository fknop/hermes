use crate::{
    edge_direction::EdgeDirection,
    geopoint::GeoPoint,
    graph_edge::GraphEdge,
    types::{EdgeId, NodeId},
};

pub trait Graph {
    type EdgeIterator<'a>: Iterator<Item = EdgeId>
    where
        Self: 'a;

    type Edge: GraphEdge;

    fn edge_count(&self) -> usize;
    fn node_count(&self) -> usize;
    fn is_virtual_node(&self, node: usize) -> bool;

    // fn node_edges(&self, node_id: usize) -> &[usize];
    fn node_edges_iter(&self, node_id: NodeId) -> Self::EdgeIterator<'_>;
    fn edge(&self, edge_id: EdgeId) -> &Self::Edge;
    fn edge_geometry(&self, edge_id: EdgeId) -> &[GeoPoint];
    fn node_geometry(&self, node_id: NodeId) -> &GeoPoint;

    fn edge_direction(&self, edge_id: EdgeId, start_node_id: NodeId) -> EdgeDirection;
}
