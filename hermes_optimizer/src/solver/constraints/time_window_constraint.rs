use jiff::Timestamp;

use crate::{
    problem::time_window::TimeWindow,
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
    fn compute_score(&self, activity: &WorkingSolutionRouteActivity) -> Score {
        TimeWindowConstraint::compute_time_window_score(
            activity.service().time_window(),
            activity.arrival_time(),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let activity = context.inserted_activity();
        let service = problem.service(activity.service_id);

        TimeWindowConstraint::compute_time_window_score(
            service.time_window(),
            activity.arrival_time,
        )
    }
}
