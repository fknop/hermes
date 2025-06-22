use crate::problem::{capacity::Capacity, vehicle_routing_problem::VehicleRoutingProblem};

use super::{
    constraint::{Constraint, RouteConstraint},
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

pub struct RouteCapacityConstraint {}

impl RouteCapacityConstraint {}

impl Constraint for RouteCapacityConstraint {
    fn constraint_name(&self) -> &'static str {
        "capacity"
    }
}

impl RouteConstraint for RouteCapacityConstraint {
    fn initialize(&mut self, route: &WorkingSolutionRoute) {}

    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score {
        let vehicle = route.vehicle();
        if vehicle.capacity().satisfies_demand(route.total_demand()) {
            Score::zero()
        } else {
            Score::hard(
                vehicle
                    .capacity()
                    .over_capacity_demand(route.total_demand())
                    .round() as i64,
            )
        }
    }
}
