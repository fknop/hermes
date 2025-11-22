use crate::solver::{
    insertion::Insertion, insertion_context::InsertionContext, score::Score,
    score_level::ScoreLevel, solution::working_solution::WorkingSolution,
};

use super::global_constraint::GlobalConstraint;

pub struct TransportCostConstraint;

pub const TRANSPORT_COST_WEIGHT: f64 = 70.0;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Soft;

impl GlobalConstraint for TransportCostConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let problem = solution.problem();
        let mut cost = 0.0;
        for route in solution.non_empty_routes_iter() {
            let vehicle = route.vehicle(problem);

            let activities = route.activities();

            if let Some(depot_location_id) = vehicle.depot_location_id() {
                cost += problem.travel_cost(
                    depot_location_id,
                    activities[0].service(problem).location_id(),
                );

                if vehicle.should_return_to_depot() {
                    cost += problem.travel_cost(
                        activities[activities.len() - 1]
                            .service(problem)
                            .location_id(),
                        depot_location_id,
                    )
                }
            }

            for (index, activity) in activities.iter().enumerate() {
                if index == 0 {
                    // Skip the first activity, as it is already counted with the depot
                    continue;
                }

                cost += problem.travel_cost(
                    activities[index - 1].service(problem).location_id(),
                    activity.service(problem).location_id(),
                )
            }
        }

        Score::of(self.score_level(), cost * TRANSPORT_COST_WEIGHT)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let service_id = context.insertion.service_id();
        let service = problem.service(service_id);

        let vehicle = match context.insertion {
            Insertion::ExistingRoute(existing_route) => context
                .solution
                .route(existing_route.route_id)
                .vehicle(problem),
            Insertion::NewRoute(new_route) => problem.vehicle(new_route.vehicle_id),
        };

        let route = match context.insertion {
            Insertion::ExistingRoute(existing_route) => {
                Some(context.solution.route(existing_route.route_id))
            }
            Insertion::NewRoute(_) => None,
        };

        let depot_location_id = vehicle.depot_location_id();

        let mut previous_location_id = None;
        let mut next_location_id = None;

        let position = context.insertion.position();

        match route {
            None => {
                if let Some(depot_id) = depot_location_id {
                    previous_location_id = Some(depot_id);

                    if vehicle.should_return_to_depot() {
                        next_location_id = Some(depot_id);
                    }
                }
            }
            Some(route) if route.is_empty() => {
                if let Some(depot_id) = depot_location_id {
                    previous_location_id = Some(depot_id);

                    if vehicle.should_return_to_depot() {
                        next_location_id = Some(depot_id);
                    }
                }
            }
            Some(route) if position == 0 => {
                if let Some(depot_id) = depot_location_id {
                    previous_location_id = Some(depot_id);
                }

                let activities = route.activities();
                next_location_id = Some(activities[0].service(problem).location_id());
            }
            Some(route) if position >= route.activities().len() => {
                // Inserting at the end
                let activities = route.activities();
                previous_location_id = Some(
                    activities[activities.len() - 1]
                        .service(problem)
                        .location_id(),
                );

                if let Some(depot_id) = depot_location_id
                    && vehicle.should_return_to_depot()
                {
                    next_location_id = Some(depot_id);
                }
            }
            Some(route) => {
                let activities = route.activities();
                previous_location_id =
                    Some(activities[position - 1].service(problem).location_id());
                next_location_id = Some(activities[position].service(problem).location_id());
            }
        }

        let old_cost =
            if let (Some(previous), Some(next)) = (previous_location_id, next_location_id) {
                problem.travel_cost(previous, next)
            } else {
                0.0
            };

        let mut new_cost = 0.0;

        if let Some(previous) = previous_location_id {
            new_cost += problem.travel_cost(previous, service.location_id());
        }

        if let Some(next) = next_location_id {
            new_cost += problem.travel_cost(service.location_id(), next);
        }

        let travel_cost_delta = new_cost - old_cost;

        Score::of(
            self.score_level(),
            travel_cost_delta * TRANSPORT_COST_WEIGHT,
        )
    }
}
