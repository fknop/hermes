use std::f64;

use crate::constants::{DISTANCE_INFLUENCE, MAX_DURATION, MAX_WEIGHT};
use crate::edge_direction::EdgeDirection;
use crate::graph::Graph;
use crate::graph_edge::GraphEdge;
use crate::properties::property::Property;

pub type Weight = u32;
pub type Milliseconds = u32;

pub trait Weighting<G>
where
    G: Graph,
{
    fn can_access_edge(&self, edge: &G::Edge) -> bool {
        self.calc_edge_weight(edge, EdgeDirection::Forward) != MAX_WEIGHT
            || self.calc_edge_weight(edge, EdgeDirection::Backward) != MAX_WEIGHT
    }

    fn calc_edge_weight(&self, edge: &G::Edge, direction: EdgeDirection) -> Weight;
    fn calc_edge_ms(&self, edge: &G::Edge, direction: EdgeDirection) -> Milliseconds;
}

#[derive(Default)]
pub struct CarWeighting<G> {
    _phantom: std::marker::PhantomData<G>,
}

impl<G: Graph> CarWeighting<G> {
    pub fn new() -> Self {
        CarWeighting {
            _phantom: std::marker::PhantomData,
        }
    }
    fn speed(edge: &G::Edge, direction: EdgeDirection) -> f32 {
        let access = edge
            .properties()
            .get_bool(Property::CarVehicleAccess, direction)
            .unwrap_or(false);

        if !access {
            return 0.0;
        }

        edge.properties()
            .get_f32(Property::CarAverageSpeed, direction)
            .unwrap_or(0.0)
    }
}

impl<G: Graph> Weighting<G> for CarWeighting<G> {
    fn calc_edge_weight(&self, edge: &G::Edge, direction: EdgeDirection) -> Weight {
        let ms = self.calc_edge_ms(edge, direction);

        if ms == MAX_DURATION {
            return MAX_WEIGHT;
        }

        let distance_costs = edge.distance().value() * DISTANCE_INFLUENCE;
        (ms as f64 + distance_costs).round() as Weight
    }

    fn calc_edge_ms(&self, edge: &G::Edge, direction: EdgeDirection) -> Milliseconds {
        let speed = Self::speed(edge, direction);
        if speed == 0.0 {
            return MAX_DURATION;
        }

        let speed_meters_per_second = speed as f64 / 3.6;
        let ms = (edge.distance().value() / speed_meters_per_second) * 1000.0;

        ms.round() as Milliseconds
    }
}
