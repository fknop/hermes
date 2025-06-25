use crate::solver::{
    insertion_context::InsertionContext,
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute},
};

use super::capacity_constraint::CapacityConstraint;

pub trait RouteConstraint {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum RouteConstraintType {
    Capacity(CapacityConstraint),
}

impl RouteConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            RouteConstraintType::Capacity(_) => "capacity",
        }
    }
}

impl RouteConstraint for RouteConstraintType {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            RouteConstraintType::Capacity(c) => c.compute_insertion_score(context),
        }
    }
}
