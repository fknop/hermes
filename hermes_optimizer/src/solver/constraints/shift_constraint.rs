use crate::solver::{
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    insertion_context::{ActivityInsertionContext, InsertionContext},
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

use super::route_constraint::RouteConstraint;

pub struct ShiftConstraint;

impl RouteConstraint for ShiftConstraint {
    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score {
        let vehicle = route.vehicle();

        if let Some(latest_end) = vehicle.latest_end_time()
            && route.end() > latest_end
        {
            Score::hard(route.end().as_second() - latest_end.as_second())
        } else {
            Score::zero()
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.solution.problem();

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle();

                if let Some(latest_end) = vehicle.latest_end_time()
                    && route.end() > latest_end
                {
                    Score::hard(context.end.as_second() - latest_end.as_second())
                } else {
                    Score::zero()
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);

                if let Some(latest_end) = vehicle.latest_end_time()
                    && context.end > latest_end
                {
                    Score::hard(context.end.as_second() - latest_end.as_second())
                } else {
                    Score::zero()
                }
            }
        }
    }
}
