use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::Insertion, insertion_context::InsertionContext, score::Score,
        score_level::ScoreLevel, solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

#[derive(Clone)]
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
        if !problem.has_waiting_duration_cost() || !problem.has_time_windows() {
            return Score::zero();
        }

        let waiting_duration = route.total_waiting_duration();

        Score::of(
            self.score_level(),
            problem.waiting_duration_cost(waiting_duration),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        if !context.problem.has_waiting_duration_cost() || !context.problem.has_time_windows() {
            return Score::zero();
        }

        let route = context.route();

        let delta = match context.insertion {
            Insertion::Service(insertion) => route.waiting_duration_change_delta(
                context.problem(),
                insertion.inserted_activity_ids(),
                insertion.position,
                insertion.position,
            ),
            Insertion::Shipment(insertion) => route.waiting_duration_change_delta(
                context.problem(),
                insertion.inserted_activity_ids(route),
                insertion.pickup_position,
                insertion.delivery_position,
            ),
        };

        Score::of(
            self.score_level(),
            context.problem.waiting_duration_cost(delta),
        )
    }
}
