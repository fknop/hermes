use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::Insertion, insertion_context::InsertionContext, score::Score,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct VehicleCostConstraint;

impl RouteConstraint for VehicleCostConstraint {
    fn compute_score(&self, problem: &VehicleRoutingProblem, _: &WorkingSolutionRoute) -> Score {
        Score::soft(problem.route_costs())
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match context.insertion {
            Insertion::ExistingRoute(_) => Score::soft(0.0),
            Insertion::NewRoute(_) => Score::soft(context.problem().route_costs()),
        }
    }
}
