use crate::{base_graph::GraphEdge, geopoint::GeoPoint, properties::property_map::EdgeDirection};

pub trait Graph {
    type EdgeIterator<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn edge_count(&self) -> usize;
    fn node_count(&self) -> usize;

    // fn node_edges(&self, node_id: usize) -> &[usize];
    fn node_edges_iter(&self, node_id: usize) -> Self::EdgeIterator<'_>;
    fn edge(&self, edge_id: usize) -> &GraphEdge;
    fn edge_geometry(&self, edge_id: usize) -> &[GeoPoint];

    fn edge_direction(&self, edge_id: usize, start_node_id: usize) -> EdgeDirection;
}
