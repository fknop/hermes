use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
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
        let route = context.route();

        if route.is_empty() {
            Score::of(self.score_level(), context.problem().fixed_vehicle_costs())
        } else {
            Score::zero()
        }
    }
}
