use jiff::Timestamp;

use crate::{
    problem::{time_window::TimeWindow, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        insertion_context::{ActivityInsertionContext, InsertionContext},
        score::Score,
        working_solution::{WorkingSolution, WorkingSolutionRouteActivity},
    },
};

use super::activity_constraint::ActivityConstraint;

pub struct TimeWindowConstraint;

impl TimeWindowConstraint {
    fn compute_time_window_score(time_windows: &Vec<TimeWindow>, arrival_time: Timestamp) -> Score {
        if time_windows.is_empty() {
            return Score::zero();
        }

        let overtime = time_windows
            .iter()
            .filter(|time_window| !time_window.is_satisfied(arrival_time))
            .map(|time_window| time_window.overtime(arrival_time))
            .min();

        Score::hard(overtime.unwrap_or(0))
    }
}

impl ActivityConstraint for TimeWindowConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        activity: &WorkingSolutionRouteActivity,
    ) -> Score {
        TimeWindowConstraint::compute_time_window_score(
            activity.service(problem).time_windows(),
            activity.arrival_time(),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        let mut total_score = Score::zero();
        for i in context.insertion.position()..context.activities.len() {
            let activity = &context.activities[i];
            let service = problem.service(activity.service_id);

            total_score += TimeWindowConstraint::compute_time_window_score(
                service.time_windows(),
                activity.arrival_time,
            )
        }

        total_score
    }
}
