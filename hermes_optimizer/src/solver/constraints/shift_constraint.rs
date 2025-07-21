use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct ShiftConstraint;

impl RouteConstraint for ShiftConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);

        if let Some(latest_end) = vehicle.latest_end_time()
            && route.end(problem) > latest_end
        {
            Score::hard((route.end(problem).as_second() - latest_end.as_second()) as f64)
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
                    && route.end(problem) > latest_end
                {
                    Score::hard((context.end.as_second() - latest_end.as_second()) as f64)
                } else {
                    Score::zero()
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);

                if let Some(latest_end) = vehicle.latest_end_time()
                    && context.end > latest_end
                {
                    Score::hard((context.end.as_second() - latest_end.as_second()) as f64)
                } else {
                    Score::zero()
                }
            }
        }
    }
}
