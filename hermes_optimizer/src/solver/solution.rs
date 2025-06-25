use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity, service::ServiceId, travel_cost_matrix::Cost, vehicle::VehicleId,
};

use super::score::Score;
#[derive(Debug)]
pub struct SolutionActivity {
    pub service_id: ServiceId,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    pub service_duration: SignedDuration,
    pub waiting_time: SignedDuration,
}

#[derive(Debug)]
pub struct SolutionRoute {
    pub vehicle_id: VehicleId,
    pub activities: Vec<SolutionActivity>,
    pub total_demand: Capacity,
    pub total_cost: Cost,
}

#[derive(Debug)]
pub struct Solution {
    pub score: Score,
    pub routes: Vec<SolutionRoute>,
}

impl Solution {}
