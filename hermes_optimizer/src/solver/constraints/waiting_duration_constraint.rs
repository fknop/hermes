use jiff::SignedDuration;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct WaitingDurationConstraint;

pub const WAITING_DURATION_WEIGHT: i64 = 1;

impl RouteConstraint for WaitingDurationConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let waiting_duration = route.total_waiting_duration();
        Score::soft(problem.waiting_cost(waiting_duration).round() as i64)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let total_waiting_duration: SignedDuration = context
            .activities
            .iter()
            .map(|activity| activity.waiting_duration)
            .sum();

        Score::soft(
            context
                .solution
                .problem()
                .waiting_cost(total_waiting_duration)
                .round() as i64,
        )
    }
}
