use jiff::SignedDuration;

use crate::{
    define_index_newtype,
    problem::{
        location::LocationIdx,
        meters::Meters,
        travel_cost_matrix::{Cost, TravelMatrices},
    },
};

define_index_newtype!(VehicleProfileIdx, VehicleProfile);

pub struct VehicleProfile {
    #[allow(dead_code)]
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
    pub fn travel_distance(&self, from: LocationIdx, to: LocationIdx) -> Meters {
        self.travel_costs.travel_distance(from, to)
    }

    #[inline(always)]
    pub fn travel_time(&self, from: LocationIdx, to: LocationIdx) -> SignedDuration {
        self.travel_costs.travel_time(from, to)
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: LocationIdx, to: LocationIdx) -> Cost {
        self.travel_costs.travel_cost(from, to)
    }

    #[inline(always)]
    pub fn travel_cost_or_zero(&self, from: Option<LocationIdx>, to: Option<LocationIdx>) -> Cost {
        if let (Some(from), Some(to)) = (from, to) {
            self.travel_cost(from, to)
        } else {
            0.0
        }
    }

    pub fn travel_costs(&self) -> &TravelMatrices {
        &self.travel_costs
    }
}
