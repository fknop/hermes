use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::Insertion, insertion_context::InsertionContext, score::Score,
        score_level::ScoreLevel, solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Soft;

pub struct VehicleCostConstraint;

impl RouteConstraint for VehicleCostConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, problem: &VehicleRoutingProblem, _: &WorkingSolutionRoute) -> Score {
        Score::soft(problem.fixed_vehicle_costs())
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match context.insertion {
            Insertion::ExistingRoute(_) => Score::of(self.score_level(), 0.0),
            Insertion::NewRoute(_) => {
                Score::of(self.score_level(), context.problem().fixed_vehicle_costs())
            }
        }
    }
}
