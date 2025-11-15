use jiff::{SignedDuration, Timestamp};

use crate::problem::{
    capacity::Capacity, service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem,
};

use super::{insertion::Insertion, working_solution::WorkingSolution};

pub struct ActivityInsertionContext {
    pub service_id: ServiceId,
    pub arrival_time: Timestamp,
    pub departure_time: Timestamp,
    pub waiting_duration: SignedDuration,
    // pub cumulative_load: Capacity,
}

impl ActivityInsertionContext {
    pub fn departure_time(&self) -> Timestamp {
        self.departure_time
    }
}

pub struct InsertionContext<'a> {
    pub problem: &'a VehicleRoutingProblem,
    pub solution: &'a WorkingSolution,
    pub insertion: &'a Insertion,
    pub initial_load: Capacity,
    pub activities: Vec<ActivityInsertionContext>,
    pub start: Timestamp,
    pub end: Timestamp,
    pub waiting_duration_delta: SignedDuration,
}

impl InsertionContext<'_> {
    pub fn inserted_activity(&self) -> &ActivityInsertionContext {
        &self.activities[self.insertion.position()]
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }
}
