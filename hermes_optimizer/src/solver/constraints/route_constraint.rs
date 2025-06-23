use crate::solver::{score::Score, working_solution::WorkingSolutionRoute};

pub trait RouteConstraint {
    fn compute_delta_score(&self, route: &WorkingSolutionRoute) -> Score;
}

pub enum RouteConstraintType {}

impl RouteConstraint for RouteConstraintType {
    fn compute_delta_score(&self, route: &WorkingSolutionRoute) -> Score {
        match self {
            _ => Score::zero(),
        }
    }
}
