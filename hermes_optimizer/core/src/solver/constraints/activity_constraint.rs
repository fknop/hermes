use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::{RouteActivityInfo, WorkingSolutionRoute},
    },
};

use super::time_window_constraint::TimeWindowConstraint;

pub trait ActivityConstraint {
    fn score_level(&self) -> ScoreLevel;
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
        activity: &RouteActivityInfo,
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
    fn score_level(&self) -> ScoreLevel {
        match self {
            Self::TimeWindow(constraint) => constraint.score_level(),
        }
    }
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            Self::TimeWindow(constraint) => constraint.compute_insertion_score(context),
        }
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
        activity: &RouteActivityInfo,
    ) -> Score {
        match self {
            Self::TimeWindow(constraint) => constraint.compute_score(problem, route, activity),
        }
    }
}
