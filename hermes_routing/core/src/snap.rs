use crate::{
    constants::INVALID_NODE,
    distance::{Distance, Meters},
    geopoint::GeoPoint,
    types::{EdgeId, NodeId},
};

#[derive(Debug)]
pub struct Snap {
    pub edge_id: usize,
    pub coordinates: GeoPoint,
    distance: Distance<Meters>,
    closest_node: Option<NodeId>,
}

impl Snap {
    pub fn new(edge_id: EdgeId, coordinates: GeoPoint, distance: Distance<Meters>) -> Self {
        Snap {
            edge_id,
            coordinates,
            distance,
            closest_node: None,
        }
    }

    pub fn closest_node(&self) -> NodeId {
        match self.closest_node {
            Some(node) => node,
            None => INVALID_NODE,
        }
    }

    pub fn set_closest_node(&mut self, node_id: NodeId) {
        self.closest_node = Some(node_id)
    }
}
