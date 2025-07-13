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
    fn compute_time_window_score(
        time_window: Option<&TimeWindow>,
        arrival_time: Timestamp,
    ) -> Score {
        if let Some(time_window) = time_window
            && !time_window.is_satisfied(arrival_time)
        {
            Score::hard(time_window.overtime(arrival_time))
        } else {
            Score::zero()
        }
    }
}

impl ActivityConstraint for TimeWindowConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        activity: &WorkingSolutionRouteActivity,
    ) -> Score {
        TimeWindowConstraint::compute_time_window_score(
            activity.service(problem).time_window(),
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
                service.time_window(),
                activity.arrival_time,
            )
        }

        total_score
    }
}
