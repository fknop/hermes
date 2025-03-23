use crate::{constants::INVALID_NODE, geopoint::GeoPoint};

#[derive(Debug)]
pub struct Snap {
    pub edge_id: usize,
    pub coordinates: GeoPoint,
    pub distance: f64,
    closest_node: Option<usize>,
}

impl Snap {
    pub fn new(edge_id: usize, coordinates: GeoPoint, distance: f64) -> Self {
        Snap {
            edge_id,
            coordinates,
            distance,
            closest_node: None,
        }
    }

    pub fn closest_node(&self) -> usize {
        match self.closest_node {
            Some(node) => node,
            None => INVALID_NODE,
        }
    }

    pub fn set_closest_node(&mut self, node_id: usize) {
        self.closest_node = Some(node_id)
    }
}
