use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct MaximumWorkingDurationConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

impl RouteConstraint for MaximumWorkingDurationConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        if let Some(maximum_working_duration) = vehicle.maximum_working_duration() {
            let working_duration = route.end(problem).duration_since(route.start(problem));
            if working_duration > maximum_working_duration {
                return Score::of(
                    self.score_level(),
                    working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                );
            }
        }

        Score::zero()
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let route = context.insertion.route(context.solution);
        let vehicle = route.vehicle(problem);

        if vehicle.maximum_working_duration().is_none() {
            return Score::zero();
        }

        let new_start = context.compute_vehicle_start();
        let new_end = context.compute_vehicle_end();
        let new_working_duration = new_end.duration_since(new_start);

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle(problem);

                // && working_duration > maximum_working_duration
                if let Some(maximum_working_duration) = vehicle.maximum_working_duration() {
                    let current_working_duration =
                        route.end(problem).duration_since(route.start(problem));

                    // New violation, old route was not violating the constraint
                    if new_working_duration > maximum_working_duration
                        && current_working_duration <= maximum_working_duration
                    {
                        return Score::of(
                            self.score_level(),
                            new_working_duration.as_secs_f64()
                                - maximum_working_duration.as_secs_f64(),
                        );

                        // Both are violating the constraint, we compute the delta between the two
                    } else if current_working_duration > maximum_working_duration
                        && new_working_duration > maximum_working_duration
                    {
                        return Score::of(
                            self.score_level(),
                            (new_working_duration - current_working_duration).as_secs_f64(),
                        );
                        // Current duration is violating, new one is not
                    } else if current_working_duration > maximum_working_duration
                        && new_working_duration <= maximum_working_duration
                    {
                        return Score::of(
                            self.score_level(),
                            (maximum_working_duration - current_working_duration).as_secs_f64(),
                        );
                    } else {
                        return Score::zero();
                    }
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);
                if let Some(maximum_working_duration) = vehicle.maximum_working_duration()
                    && new_working_duration > maximum_working_duration
                {
                    return Score::of(
                        self.score_level(),
                        new_working_duration.as_secs_f64() - maximum_working_duration.as_secs_f64(),
                    );
                }
            }
        }

        Score::zero()
    }
}
