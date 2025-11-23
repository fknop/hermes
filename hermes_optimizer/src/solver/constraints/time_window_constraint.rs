use jiff::Timestamp;

use crate::{
    problem::{time_window::TimeWindow, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::{activity::WorkingSolutionRouteActivity, route::WorkingSolutionRoute},
    },
};

use super::activity_constraint::ActivityConstraint;

pub struct TimeWindowConstraint {
    score_level: ScoreLevel,
}

impl Default for TimeWindowConstraint {
    fn default() -> Self {
        TimeWindowConstraint {
            score_level: ScoreLevel::Hard,
        }
    }
}

impl TimeWindowConstraint {
    pub fn new(score_level: ScoreLevel) -> Self {
        TimeWindowConstraint { score_level }
    }
}

impl TimeWindowConstraint {
    pub fn compute_time_window_score(
        level: ScoreLevel,
        time_windows: &[TimeWindow],
        arrival_time: Timestamp,
    ) -> Score {
        if time_windows.is_empty() {
            return Score::zero();
        }

        // If at least one time window is satisfied, the constraint is satisfied
        if time_windows.iter().any(|tw| tw.is_satisfied(arrival_time)) {
            return Score::zero();
        }

        let overtime = time_windows
            .iter()
            .filter(|time_window| !time_window.is_satisfied(arrival_time))
            .map(|time_window| time_window.overtime(arrival_time))
            .min();

        Score::of(level, overtime.unwrap_or(0) as f64)
    }
}

impl ActivityConstraint for TimeWindowConstraint {
    fn score_level(&self) -> ScoreLevel {
        self.score_level
    }
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        _route: &WorkingSolutionRoute,
        activity: &WorkingSolutionRouteActivity,
    ) -> Score {
        TimeWindowConstraint::compute_time_window_score(
            self.score_level(),
            activity.service(problem).time_windows(),
            activity.arrival_time(),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        let route = context.insertion.route(context.solution);
        if route.is_valid_tw_change(
            problem,
            [context.insertion.service_id()].into_iter(),
            context.insertion.position(),
            context.insertion.position(),
        ) {
            return Score::zero();
        }

        let mut total_score = Score::zero();

        // TODO: precompute time slacks to avoid recomputing for all activities
        for i in context.insertion.position()..context.activities.len() {
            let activity = &context.activities[i];
            let service = problem.service(activity.service_id);

            total_score += TimeWindowConstraint::compute_time_window_score(
                self.score_level(),
                service.time_windows(),
                activity.arrival_time,
            )
        }

        total_score
    }
}
