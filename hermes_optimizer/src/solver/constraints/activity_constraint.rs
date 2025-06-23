use crate::solver::{score::Score, working_solution::WorkingSolutionRouteActivity};

use super::time_window_constraint::TimeWindowConstraint;

pub trait ActivityConstraint {
    fn compute_delta_score(&self, activity: &WorkingSolutionRouteActivity) -> Score;
}

pub enum ActivityConstraintType {
    TimeWindow(TimeWindowConstraint),
}

impl ActivityConstraint for ActivityConstraintType {
    fn compute_delta_score(&self, activity: &WorkingSolutionRouteActivity) -> Score {
        match self {
            _ => Score::zero(),
        }
    }
}
