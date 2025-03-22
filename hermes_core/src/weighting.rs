use crate::graph::GraphEdge;
use crate::properties::property::Property;
use crate::properties::property_map::EdgeDirection;

pub trait Weighting {
    fn calc_edge_weight(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize;
    fn calc_edge_ms(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize;
}

pub struct CarWeighting;

impl CarWeighting {
    pub fn new() -> Self {
        CarWeighting {}
    }
    fn speed(edge: &GraphEdge, direction: EdgeDirection) -> u8 {
        let access = edge
            .properties
            .get_bool(Property::VehicleAccess("car".to_string()), direction)
            .unwrap_or(false);

        if access == false {
            return 0;
        }

        let speed = edge.properties.get_u8(Property::MaxSpeed, direction);

        speed.unwrap_or(0)
    }
}

impl Weighting for CarWeighting {
    fn calc_edge_weight(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize {
        let speed = Self::speed(edge, direction);
        if speed == 0 {
            return usize::MAX;
        }

        let speed_meters_per_second = (speed as f64) * (1000.0 / 3600.0);
        (edge.distance() / speed_meters_per_second).round() as usize
    }

    fn calc_edge_ms(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize {
        let time = self.calc_edge_weight(edge, direction) * 1000;
        time
    }
}
