use crate::solver::{
    insertion_context::InsertionContext,
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute},
};

use super::{capacity_constraint::CapacityConstraint, shift_constraint::ShiftConstraint};

pub trait RouteConstraint {
    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score;
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum RouteConstraintType {
    Capacity(CapacityConstraint),
    Shift(ShiftConstraint),
}

impl RouteConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            RouteConstraintType::Capacity(_) => "capacity",
            RouteConstraintType::Shift(_) => "shift",
        }
    }
}

impl RouteConstraint for RouteConstraintType {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_insertion_score(context),
            RouteConstraintType::Shift(s) => s.compute_insertion_score(context),
        }
    }

    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_score(route),
            RouteConstraintType::Shift(s) => s.compute_score(route),
        }
    }
}
