use crate::problem::{capacity::Capacity, vehicle_routing_problem::VehicleRoutingProblem};

use super::{
    constraint::{Constraint, RouteConstraint},
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

pub struct RouteCapacityConstraint {
    total_demand: Capacity,
}

impl RouteCapacityConstraint {}

impl Constraint for RouteCapacityConstraint {
    fn constraint_name(&self) -> &'static str {
        "capacity"
    }

    fn service_inserted(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        route: &WorkingSolutionRoute,
        activity: &WorkingSolutionRouteActivity,
    ) {
        self.total_demand.add(activity.service().demand());
    }

    fn service_removed(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        route: &WorkingSolutionRoute,
        activity: &WorkingSolutionRouteActivity,
    ) {
        self.total_demand.sub(activity.service().demand());
    }
}

impl RouteConstraint for RouteCapacityConstraint {
    fn initialize(&mut self, route: &WorkingSolutionRoute) {
        self.total_demand.reset();
        for activity in route.activities() {
            self.total_demand.add(activity.service().demand());
        }
    }

    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score {
        let vehicle = route.vehicle();
        if vehicle.capacity().satisfies_demand(&self.total_demand) {
            Score::zero()
        } else {
            Score::hard(
                vehicle
                    .capacity()
                    .over_capacity_demand(&self.total_demand)
                    .round() as i64,
            )
        }
    }
}
