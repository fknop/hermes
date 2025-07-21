use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, working_solution::WorkingSolutionRoute,
    },
};

use super::{
    capacity_constraint::CapacityConstraint,
    maximum_working_duration_constraint::MaximumWorkingDurationConstraint,
    shift_constraint::ShiftConstraint, vehicle_cost_constraint::VehicleCostConstraint,
    waiting_duration_constraint::WaitingDurationConstraint,
};

pub trait RouteConstraint {
    fn compute_score(&self, problem: &VehicleRoutingProblem, route: &WorkingSolutionRoute)
    -> Score;
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum RouteConstraintType {
    Capacity(CapacityConstraint),
    Shift(ShiftConstraint),
    MaximumWorkingDuration(MaximumWorkingDurationConstraint),
    WaitingDuration(WaitingDurationConstraint),
    VehicleCost(VehicleCostConstraint),
}

impl RouteConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            RouteConstraintType::Capacity(_) => "capacity",
            RouteConstraintType::Shift(_) => "shift",
            RouteConstraintType::WaitingDuration(_) => "waiting_duration",
            RouteConstraintType::VehicleCost(_) => "vehicle_cost",
            RouteConstraintType::MaximumWorkingDuration(_) => "maximum_working_duration",
        }
    }
}

impl RouteConstraint for RouteConstraintType {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_insertion_score(context),
            RouteConstraintType::Shift(s) => s.compute_insertion_score(context),
            RouteConstraintType::WaitingDuration(w) => w.compute_insertion_score(context),
            RouteConstraintType::VehicleCost(v) => v.compute_insertion_score(context),
            RouteConstraintType::MaximumWorkingDuration(m) => m.compute_insertion_score(context),
        }
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_score(problem, route),
            RouteConstraintType::Shift(s) => s.compute_score(problem, route),
            RouteConstraintType::WaitingDuration(w) => w.compute_score(problem, route),
            RouteConstraintType::VehicleCost(v) => v.compute_score(problem, route),
            RouteConstraintType::MaximumWorkingDuration(m) => m.compute_score(problem, route),
        }
    }
}
