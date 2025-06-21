use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

pub trait Constraint {
    fn constraint_name(&self) -> &'static str;

    fn service_inserted(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        route: &WorkingSolutionRoute,
        activity: &WorkingSolutionRouteActivity,
    ) {
    }

    fn service_removed(
        &mut self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        route: &WorkingSolutionRoute,
        activity: &WorkingSolutionRouteActivity,
    ) {
    }
}

pub trait GlobalConstraint {
    fn compute_score(&self, solution: &WorkingSolution) -> Score;
}

pub trait RouteConstraint {
    fn initialize(&mut self, route: &WorkingSolutionRoute);
    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score;
}
