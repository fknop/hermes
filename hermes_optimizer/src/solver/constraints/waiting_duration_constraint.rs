use jiff::SignedDuration;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct WaitingDurationConstraint;

pub const WAITING_DURATION_WEIGHT: i64 = 1;
const SCORE_LEVEL: ScoreLevel = ScoreLevel::Soft;

impl RouteConstraint for WaitingDurationConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

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

        Score::of(self.score_level(), problem.waiting_cost(waiting_duration))
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

        Score::of(
            self.score_level(),
            context
                .solution
                .problem()
                .waiting_cost(total_waiting_duration),
        )
    }
}
