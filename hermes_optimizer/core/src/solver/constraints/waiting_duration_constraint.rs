use jiff::SignedDuration;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct WaitingDurationConstraint;

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
        if !problem.has_waiting_duration_cost() {
            return Score::zero();
        }

        let waiting_duration = route
            .activity_ids()
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let activity = route.activity(index);
                if activity.waiting_duration().as_secs()
                    > problem.acceptable_service_waiting_duration_secs()
                {
                    activity.waiting_duration()
                } else {
                    SignedDuration::ZERO
                }
            })
            .sum();

        Score::of(
            self.score_level(),
            problem.waiting_duration_cost(waiting_duration),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        if !context.problem.has_waiting_duration_cost() || !context.problem.has_time_windows() {
            return Score::zero();
        }

        let mut delta = SignedDuration::ZERO;

        for activity_info in context.updated_activities_iter() {
            let new_waiting_duration = activity_info.waiting_duration;
            let old_waiting_duration = activity_info
                .current_position
                .map(|position| context.route().waiting_duration(position))
                .unwrap_or(SignedDuration::ZERO);

            delta += new_waiting_duration;
            delta -= old_waiting_duration;
        }

        Score::of(
            self.score_level(),
            context.solution.problem().waiting_duration_cost(delta),
        )
    }
}
