use std::ops::Add;

use crate::{
    problem::{capacity::Capacity, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct CapacityConstraint;

impl CapacityConstraint {
    fn compute_capacity_score(
        vehicle_capacity: &Capacity,
        initial_load: &Capacity,
        cumulative_load: &Capacity,
    ) -> Score {
        let load = initial_load.add(cumulative_load);
        if vehicle_capacity.satisfies_demand(&load) {
            Score::zero()
        } else {
            Score::hard(vehicle_capacity.over_capacity_demand(&load))
        }
    }
}

impl RouteConstraint for CapacityConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        let initial_load = route.total_initial_load();

        let mut score = Score::zero();
        if !vehicle.capacity().satisfies_demand(initial_load) {
            score += Score::hard(vehicle.capacity().over_capacity_demand(initial_load));
        }

        for activity in route.activities() {
            score += CapacityConstraint::compute_capacity_score(
                vehicle.capacity(),
                initial_load,
                activity.cumulative_load(),
            );
        }

        score
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let service = problem.service(context.insertion.service_id());

        if service.demand().is_empty() {
            return Score::zero();
        }

        let vehicle = match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                context.solution.route(route_id).vehicle(problem)
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                problem.vehicle(vehicle_id)
            }
        };

        let mut score = Score::zero();
        if !vehicle.capacity().satisfies_demand(&context.initial_load) {
            score += Score::hard(
                vehicle
                    .capacity()
                    .over_capacity_demand(&context.initial_load),
            );
        }

        for activity in context.activities.iter().skip(context.insertion.position()) {
            score += CapacityConstraint::compute_capacity_score(
                vehicle.capacity(),
                &context.initial_load,
                &activity.cumulative_load,
            );
        }

        score
    }
}
