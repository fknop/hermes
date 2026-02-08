use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::maximum_activities_constraint::MaximumActivitiesConstraint,
        insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
    },
};

use super::{
    capacity_constraint::CapacityConstraint,
    maximum_working_duration_constraint::MaximumWorkingDurationConstraint,
    shift_constraint::ShiftConstraint, vehicle_cost_constraint::VehicleCostConstraint,
    waiting_duration_constraint::WaitingDurationConstraint,
};

pub trait RouteConstraint {
    fn score_level(&self) -> ScoreLevel;

    fn compute_score(&self, problem: &VehicleRoutingProblem, route: &WorkingSolutionRoute)
    -> Score;
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

#[derive(Clone)]
pub enum RouteConstraintType {
    Capacity(CapacityConstraint),
    Shift(ShiftConstraint),
    MaximumWorkingDuration(MaximumWorkingDurationConstraint),
    WaitingDuration(WaitingDurationConstraint),
    VehicleCost(VehicleCostConstraint),
    MaximumJobs(MaximumActivitiesConstraint),
}

impl RouteConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            RouteConstraintType::Capacity(_) => "capacity",
            RouteConstraintType::Shift(_) => "shift",
            RouteConstraintType::WaitingDuration(_) => "waiting_duration",
            RouteConstraintType::VehicleCost(_) => "vehicle_cost",
            RouteConstraintType::MaximumWorkingDuration(_) => "maximum_working_duration",
            RouteConstraintType::MaximumJobs(_) => "maximum_activities",
        }
    }
}

impl RouteConstraint for RouteConstraintType {
    fn score_level(&self) -> ScoreLevel {
        match self {
            RouteConstraintType::Capacity(c) => c.score_level(),
            RouteConstraintType::Shift(c) => c.score_level(),
            RouteConstraintType::WaitingDuration(c) => c.score_level(),
            RouteConstraintType::VehicleCost(c) => c.score_level(),
            RouteConstraintType::MaximumWorkingDuration(c) => c.score_level(),
            RouteConstraintType::MaximumJobs(c) => c.score_level(),
        }
    }
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_insertion_score(context),
            RouteConstraintType::Shift(c) => c.compute_insertion_score(context),
            RouteConstraintType::WaitingDuration(c) => c.compute_insertion_score(context),
            RouteConstraintType::VehicleCost(c) => c.compute_insertion_score(context),
            RouteConstraintType::MaximumWorkingDuration(c) => c.compute_insertion_score(context),
            RouteConstraintType::MaximumJobs(c) => c.compute_insertion_score(context),
        }
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_score(problem, route),
            RouteConstraintType::Shift(c) => c.compute_score(problem, route),
            RouteConstraintType::WaitingDuration(c) => c.compute_score(problem, route),
            RouteConstraintType::VehicleCost(c) => c.compute_score(problem, route),
            RouteConstraintType::MaximumWorkingDuration(c) => c.compute_score(problem, route),
            RouteConstraintType::MaximumJobs(c) => c.compute_score(problem, route),
        }
    }
}
