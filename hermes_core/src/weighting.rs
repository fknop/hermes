use std::f64;

use crate::base_graph::GraphEdge;
use crate::constants::MAX_WEIGHT;
use crate::edge_direction::EdgeDirection;
use crate::properties::property::Property;

pub type Weight = usize;

pub trait Weighting {
    fn can_access_edge(&self, edge: &GraphEdge) -> bool {
        self.calc_edge_weight(edge, EdgeDirection::Forward) != MAX_WEIGHT
            || self.calc_edge_weight(edge, EdgeDirection::Backward) != MAX_WEIGHT
    }

    fn calc_edge_weight(&self, edge: &GraphEdge, direction: EdgeDirection) -> Weight;
    fn calc_edge_ms(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize;
}

#[derive(Default)]
pub struct CarWeighting;

impl CarWeighting {
    pub fn new() -> Self {
        CarWeighting
    }
    fn speed(edge: &GraphEdge, direction: EdgeDirection) -> u8 {
        let access = edge
            .properties
            .get_bool(Property::CarVehicleAccess, direction)
            .unwrap_or(false);

        if !access {
            return 0;
        }

        let speed = edge.properties.get_u8(Property::CarAverageSpeed, direction);

        speed.unwrap_or(0)
    }
}

const DISTANCE_INFLUENCE: f64 = 0.7;

impl Weighting for CarWeighting {
    fn calc_edge_weight(&self, edge: &GraphEdge, direction: EdgeDirection) -> Weight {
        let ms = self.calc_edge_ms(edge, direction);

        if ms == MAX_WEIGHT {
            return MAX_WEIGHT;
        }

        let distance_costs = edge.distance().value() * DISTANCE_INFLUENCE;
        ((ms as f64 / 1000.0) + distance_costs).round() as usize
    }

    fn calc_edge_ms(&self, edge: &GraphEdge, direction: EdgeDirection) -> usize {
        let speed = Self::speed(edge, direction);
        if speed == 0 {
            return usize::MAX;
        }

        let speed_meters_per_second = (speed as f64) / 3.6;
        let ms: f64 = (edge.distance().value() / speed_meters_per_second) * 1000.0;

        ms.round() as usize
    }
}
