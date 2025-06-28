use crate::solver::{
    insertion::Insertion, insertion_context::InsertionContext, score::Score,
    working_solution::WorkingSolution,
};

use super::global_constraint::GlobalConstraint;

pub struct TransportCostConstraint;

impl GlobalConstraint for TransportCostConstraint {
    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let problem = solution.problem();
        let mut cost = 0;
        for route in solution.routes() {
            let vehicle = route.vehicle();

            let activities = route.activities();

            if let Some(depot_location_id) = vehicle.depot_location_id() {
                cost +=
                    problem.travel_cost(depot_location_id, activities[0].service().location_id());

                if vehicle.should_return_to_depot() {
                    cost += problem.travel_cost(
                        activities[activities.len() - 1].service().location_id(),
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
                    activities[index - 1].service().location_id(),
                    activity.service().location_id(),
                )
            }
        }

        Score::soft(cost)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let service_id = context.insertion.service_id();
        let service = problem.service(service_id);

        let vehicle = match context.insertion {
            Insertion::ExistingRoute(existing_route) => {
                context.solution.route(existing_route.route_id).vehicle()
            }
            Insertion::NewRoute(new_route) => problem.vehicle(new_route.vehicle_id),
        };

        let route = match context.insertion {
            Insertion::ExistingRoute(existing_route) => {
                context.solution.routes().get(existing_route.route_id)
            }
            Insertion::NewRoute(_) => None,
        };

        let depot_location_id = vehicle.depot_location_id();

        let mut previous_location_id = None;
        let mut next_location_id = None;

        let position = context.insertion.position();
        if route.is_none() || route.unwrap().is_empty() {
            if let Some(depot_id) = depot_location_id {
                previous_location_id = Some(depot_id);

                if vehicle.should_return_to_depot() {
                    next_location_id = Some(depot_id);
                }
            }
        } else if position == 0 {
            if let Some(depot_id) = depot_location_id {
                previous_location_id = Some(depot_id);
            }

            let activities = route.unwrap().activities();
            next_location_id = Some(activities[0].service().location_id());
        } else if position >= route.unwrap().activities().len() {
            // Inserting at the end
            let activities = route.unwrap().activities();
            previous_location_id = Some(activities[activities.len() - 1].service().location_id());

            if let Some(depot_id) = depot_location_id
                && vehicle.should_return_to_depot()
            {
                next_location_id = Some(depot_id);
            }
        } else {
            let activities = route.unwrap().activities();
            previous_location_id = Some(activities[position - 1].service().location_id());
            next_location_id = Some(activities[position].service().location_id());
        }

        let old_cost =
            if let (Some(previous), Some(next)) = (previous_location_id, next_location_id) {
                problem.travel_cost(previous, next)
            } else {
                0
            };

        let mut new_cost = 0;

        if let Some(previous) = previous_location_id {
            new_cost += problem.travel_cost(previous, service.location_id());
        }

        if let Some(next) = next_location_id {
            new_cost += problem.travel_cost(service.location_id(), next);
        }

        let travel_cost_delta = new_cost - old_cost;

        Score::soft(travel_cost_delta)
    }
}
