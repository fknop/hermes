use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, noise::NoiseGenerator, score::Score,
        working_solution::WorkingSolution,
    },
};

use super::{
    activity_constraint::{ActivityConstraint, ActivityConstraintType},
    global_constraint::{GlobalConstraint, GlobalConstraintType},
    route_constraint::{RouteConstraint, RouteConstraintType},
};

pub enum Constraint {
    Global(GlobalConstraintType),
    Route(RouteConstraintType),
    Activity(ActivityConstraintType),
}

impl Constraint {
    pub fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let insertion_score = match self {
            Constraint::Global(constraint) => constraint.compute_insertion_score(context),
            Constraint::Route(constraint) => constraint.compute_insertion_score(context),
            Constraint::Activity(constraint) => constraint.compute_insertion_score(context),
        };

        insertion_score
    }

    pub fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
    ) -> Score {
        match self {
            Constraint::Global(constraint) => constraint.compute_score(solution),
            Constraint::Route(constraint) => {
                solution.routes().iter().fold(Score::zero(), |acc, route| {
                    acc + constraint.compute_score(problem, route)
                })
            }
            Constraint::Activity(constraint) => {
                solution.routes().iter().fold(Score::zero(), |acc, route| {
                    acc + route
                        .activities()
                        .iter()
                        .fold(Score::zero(), |acc, activity| {
                            acc + constraint.compute_score(problem, route, activity)
                        })
                })
            }
        }
    }

    pub fn constraint_name(&self) -> &'static str {
        match self {
            Constraint::Global(c) => c.constraint_name(),
            Constraint::Route(c) => c.constraint_name(),
            Constraint::Activity(c) => c.constraint_name(),
        }
    }
}
