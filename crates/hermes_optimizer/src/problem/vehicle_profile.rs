use jiff::SignedDuration;

use crate::problem::travel_cost_matrix::{Cost, Distance, TravelMatrices};

pub struct VehicleProfile {
    external_id: String,
    travel_costs: TravelMatrices,
}

impl VehicleProfile {
    pub fn new(external_id: String, travel_costs: TravelMatrices) -> Self {
        Self {
            external_id,
            travel_costs,
        }
    }

    #[inline(always)]
    pub fn travel_distance(&self, from: usize, to: usize) -> Distance {
        self.travel_costs.travel_distance(from, to)
    }

    #[inline(always)]
    pub fn travel_time(&self, from: usize, to: usize) -> SignedDuration {
        self.travel_costs.travel_time(from, to)
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: usize, to: usize) -> Cost {
        self.travel_costs.travel_cost(from, to)
    }

    pub fn travel_costs(&self) -> &TravelMatrices {
        &self.travel_costs
    }
}
