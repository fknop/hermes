use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
        solution::working_solution::WorkingSolution,
    },
};

use super::{
    activity_constraint::{ActivityConstraint, ActivityConstraintType},
    global_constraint::{GlobalConstraint, GlobalConstraintType},
    route_constraint::{RouteConstraint, RouteConstraintType},
};

#[derive(Clone)]
pub enum Constraint {
    Global(GlobalConstraintType),
    Route(RouteConstraintType),
    Activity(ActivityConstraintType),
}

impl Constraint {
    pub fn score_level(&self) -> ScoreLevel {
        match self {
            Constraint::Global(constraint) => constraint.score_level(),
            Constraint::Route(constraint) => constraint.score_level(),
            Constraint::Activity(constraint) => constraint.score_level(),
        }
    }

    pub fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            Constraint::Global(constraint) => constraint.compute_insertion_score(context),
            Constraint::Route(constraint) => constraint.compute_insertion_score(context),
            Constraint::Activity(constraint) => constraint.compute_insertion_score(context),
        }
    }

    pub fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
    ) -> Score {
        match self {
            Constraint::Global(constraint) => constraint.compute_score(solution),
            Constraint::Route(constraint) => solution
                .non_empty_routes_iter()
                .fold(Score::zero(), |acc, route| {
                    acc + constraint.compute_score(problem, route)
                }),
            Constraint::Activity(constraint) => {
                solution
                    .non_empty_routes_iter()
                    .fold(Score::zero(), |acc, route| {
                        acc + route.activity_ids().iter().enumerate().fold(
                            Score::zero(),
                            |acc, (index, _)| {
                                acc + constraint.compute_score(
                                    problem,
                                    route,
                                    &route.activity(index),
                                )
                            },
                        )
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
