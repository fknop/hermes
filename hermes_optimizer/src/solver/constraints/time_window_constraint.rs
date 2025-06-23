use crate::solver::{score::Score, working_solution::WorkingSolutionRouteActivity};

use super::activity_constraint::ActivityConstraint;

pub struct TimeWindowConstraint;

impl ActivityConstraint for TimeWindowConstraint {
    fn compute_delta_score(&self, activity: &WorkingSolutionRouteActivity) -> Score {
        todo!()
    }
}
