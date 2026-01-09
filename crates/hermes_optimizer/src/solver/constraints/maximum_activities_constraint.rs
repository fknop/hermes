use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::route_constraint::RouteConstraint, insertion_context::InsertionContext,
        score::Score, score_level::ScoreLevel, solution::route::WorkingSolutionRoute,
    },
};

pub struct MaximumActivitiesConstraint;

const WEIGHT: f64 = 1000.0;

impl RouteConstraint for MaximumActivitiesConstraint {
    fn score_level(&self) -> ScoreLevel {
        ScoreLevel::Hard
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> crate::solver::score::Score {
        let vehicle = route.vehicle(problem);
        if let Some(maximum_activities) = vehicle.maximum_activities() {
            if route.len() > maximum_activities {
                Score::hard(WEIGHT)
            } else {
                Score::ZERO
            }
        } else {
            Score::ZERO
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let route = context.route();
        let vehicle = route.vehicle(context.problem);

        if let Some(maximum_activities) = vehicle.maximum_activities() {
            let new_len = route.len() + 1;
            if new_len > maximum_activities {
                Score::hard(WEIGHT)
            } else {
                Score::ZERO
            }
        } else {
            Score::ZERO
        }
    }
}
