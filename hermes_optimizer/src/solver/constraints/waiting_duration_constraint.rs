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
        let waiting_duration = route
            .activities()
            .iter()
            .map(|activity| {
                if activity.waiting_duration().as_secs()
                    > problem.acceptable_service_waiting_duration_secs()
                {
                    activity.waiting_duration()
                } else {
                    SignedDuration::ZERO
                }
            })
            .sum();

        Score::soft(problem.waiting_cost(waiting_duration))
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let total_waiting_duration: SignedDuration = context
            .activities
            .iter()
            .map(|activity| {
                if activity.waiting_duration.as_secs()
                    > context.problem().acceptable_service_waiting_duration_secs()
                {
                    activity.waiting_duration
                } else {
                    SignedDuration::ZERO
                }
            })
            .sum();

        Score::soft(
            context
                .solution
                .problem()
                .waiting_cost(total_waiting_duration),
        )
    }
}
