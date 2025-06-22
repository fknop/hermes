use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

pub trait GlobalConstraint {
    fn compute_delta_score(&self, solution: &WorkingSolution) -> Score;
    fn constraint_name(&self) -> &'static str;
}

pub trait RouteConstraint {
    fn compute_delta_score(&self, route: &WorkingSolutionRoute) -> Score;
    fn constraint_name(&self) -> &'static str;
}

pub trait ActivityConstraint {
    fn compute_delta_score(&self, activity: &WorkingSolutionRouteActivity) -> Score;
    fn constraint_name(&self) -> &'static str;
}

pub enum Constraint {
    Global(Box<dyn GlobalConstraint>),
    Route(Box<dyn RouteConstraint>),
    Activity(Box<dyn ActivityConstraint>),
}

impl Constraint {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            Constraint::Global(c) => c.constraint_name(),
            Constraint::Route(c) => c.constraint_name(),
            Constraint::Activity(c) => c.constraint_name(),
        }
    }
}
