use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

pub struct ShiftConstraint;

impl RouteConstraint for ShiftConstraint {
    fn score_level(&self) -> crate::solver::score_level::ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);

        if let Some(latest_end) = vehicle.latest_end_time()
            && route.end(problem) > latest_end
        {
            Score::of(
                self.score_level(),
                (route.end(problem).as_second() - latest_end.as_second()) as f64,
            )
        } else {
            Score::zero()
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle(problem);

                if let Some(latest_end) = vehicle.latest_end_time()
                    && context.end > latest_end
                {
                    Score::of(
                        self.score_level(),
                        (context.end.as_second() - latest_end.as_second()) as f64,
                    )
                } else {
                    Score::zero()
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);

                if let Some(latest_end) = vehicle.latest_end_time()
                    && context.end > latest_end
                {
                    Score::of(
                        self.score_level(),
                        (context.end.as_second() - latest_end.as_second()) as f64,
                    )
                } else {
                    Score::zero()
                }
            }
        }
    }
}
