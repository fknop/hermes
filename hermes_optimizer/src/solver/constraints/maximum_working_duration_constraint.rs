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

pub struct MaximumWorkingDurationConstraint;

impl RouteConstraint for MaximumWorkingDurationConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        if let Some(maximum_working_duration) = vehicle.maximum_working_duration() {
            let working_duration = route.end(problem).duration_since(route.start(problem));
            if working_duration > maximum_working_duration {
                return Score::hard(
                    working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                );
            }
        }

        Score::zero()
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        let working_duration = context.end.duration_since(context.start);

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle(problem);

                if let Some(maximum_working_duration) = vehicle.maximum_working_duration()
                    && working_duration > maximum_working_duration
                {
                    return Score::hard(
                        working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                    );
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);
                if let Some(maximum_working_duration) = vehicle.maximum_working_duration()
                    && working_duration > maximum_working_duration
                {
                    return Score::hard(
                        working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                    );
                }
            }
        }

        Score::zero()
    }
}
