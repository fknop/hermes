use crate::solver::{
    insertion_context::{ActivityInsertionContext, InsertionContext},
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRouteActivity},
};

use super::activity_constraint::ActivityConstraint;

pub struct TimeWindowConstraint;

impl ActivityConstraint for TimeWindowConstraint {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let activity = context.inserted_activity();
        let service = problem.service(activity.service_id);
        if let Some(time_window) = service.time_window() {
            if time_window.is_satisfied(activity.arrival_time) {
                Score::zero()
            } else {
                Score::hard(time_window.overtime(activity.arrival_time))
            }
        } else {
            Score::zero()
        }
    }
}
