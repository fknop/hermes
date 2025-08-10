use std::f64;

use crate::{
    problem::{service::ServiceId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::working_solution::WorkingSolution,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinWorst;

fn get_worst(problem: &VehicleRoutingProblem, solution: &WorkingSolution) -> Option<ServiceId> {
    let mut worst_service: Option<usize> = None;
    let mut best_savings = f64::MIN;

    for route in solution.routes() {
        let vehicle = route.vehicle(problem);
        for (index, activity) in route.activities().iter().enumerate() {
            let previous_location_id = if index == 0 {
                vehicle.depot_location_id()
            } else {
                Some(route.activities()[index - 1].service(problem).location_id())
            };

            let next_location_id = if index == route.activities().len() - 1 {
                if vehicle.should_return_to_depot() {
                    vehicle.depot_location_id()
                } else {
                    None
                }
            } else {
                Some(route.activities()[index + 1].service(problem).location_id())
            };

            let location_id = activity.service(problem).location_id();

            let travel_cost_previous = if let Some(previous_location_id) = previous_location_id {
                solution
                    .problem()
                    .travel_cost(previous_location_id, location_id)
            } else {
                0.0
            };

            let travel_cost_next = if let Some(next_location_id) = next_location_id {
                solution
                    .problem()
                    .travel_cost(location_id, next_location_id)
            } else {
                0.0
            };

            let new_travel_cost = if let Some(next_location_id) = next_location_id
                && let Some(previous_location_id) = previous_location_id
            {
                solution
                    .problem()
                    .travel_cost(previous_location_id, next_location_id)
            } else {
                0.0
            };

            let service_savings: f64 = new_travel_cost - (travel_cost_previous + travel_cost_next);

            if service_savings > best_savings {
                best_savings = service_savings;
                worst_service = Some(activity.service_id());
            }
        }
    }

    worst_service
}

impl RuinSolution for RuinWorst {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        for _ in 0..context.num_activities_to_remove {
            if solution.routes().is_empty() {
                return;
            }

            // Remove the activity with the worst savings
            if let Some(service_id) = get_worst(context.problem, solution) {
                solution.remove_service(service_id);
            } else {
                break;
            }
        }
    }
}
