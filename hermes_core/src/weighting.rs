use crate::base_graph::GraphEdge;
use crate::constants::MAX_WEIGHT;
use crate::properties::property::Property;
use crate::properties::property_map::{BACKWARD_EDGE, EdgeDirection, FORWARD_EDGE};

pub trait Weighting {
    fn can_access_edge(&self, edge: &GraphEdge) -> bool {
        self.calc_edge_weight(edge, FORWARD_EDGE) != MAX_WEIGHT
            || self.calc_edge_weight(edge, BACKWARD_EDGE) != MAX_WEIGHT
    }

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
        if edge.id() >= 95637 {
            println!(
                "Computing for virtual edge {}. Distance = {}, speed = {}",
                edge.id(),
                edge.distance(),
                speed
            )
        }
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
