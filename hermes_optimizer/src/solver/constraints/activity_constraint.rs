use crate::{
    problem::{time_window::TimeWindow, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        insertion_context::{ActivityInsertionContext, InsertionContext},
        score::Score,
        working_solution::{WorkingSolution, WorkingSolutionRouteActivity},
    },
};

use super::time_window_constraint::TimeWindowConstraint;

pub trait ActivityConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        activity: &WorkingSolutionRouteActivity,
    ) -> Score;
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum ActivityConstraintType {
    TimeWindow(TimeWindowConstraint),
}

impl ActivityConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            Self::TimeWindow(_) => "time_window",
        }
    }
}

impl ActivityConstraint for ActivityConstraintType {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            Self::TimeWindow(constraint) => constraint.compute_insertion_score(context),
        }
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        activity: &WorkingSolutionRouteActivity,
    ) -> Score {
        match self {
            Self::TimeWindow(constraint) => constraint.compute_score(problem, activity),
        }
    }
}
