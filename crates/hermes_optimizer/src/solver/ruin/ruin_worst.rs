use std::f64;

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::solution::{route::WorkingSolutionRoute, working_solution::WorkingSolution},
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments
pub struct RuinWorst;

fn compute_savings(
    problem: &VehicleRoutingProblem,
    route: &WorkingSolutionRoute,
    index: usize,
) -> f64 {
    let vehicle = route.vehicle(problem);
    let activity = &route.activity(index);
    let previous_location_id = if index == 0 {
        vehicle.depot_location_id()
    } else {
        Some(route.activity(index - 1).job_task(problem).location_id())
    };

    let next_location_id = if index == route.activity_ids().len() - 1 {
        if vehicle.should_return_to_depot() {
            vehicle.depot_location_id()
        } else {
            None
        }
    } else {
        Some(route.activity(index + 1).job_task(problem).location_id())
    };

    let location_id = activity.job_task(problem).location_id();

    let travel_cost_previous = if let Some(previous_location_id) = previous_location_id {
        problem.travel_cost(vehicle, previous_location_id, location_id)
    } else {
        0.0
    };

    let travel_cost_next = if let Some(next_location_id) = next_location_id {
        problem.travel_cost(vehicle, location_id, next_location_id)
    } else {
        0.0
    };

    let new_travel_cost = if let Some(next_location_id) = next_location_id
        && let Some(previous_location_id) = previous_location_id
    {
        problem.travel_cost(vehicle, previous_location_id, next_location_id)
    } else {
        0.0
    };

    new_travel_cost - (travel_cost_previous + travel_cost_next)
}

impl RuinSolution for RuinWorst {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let p = context.params.ruin_worst_determinism;

        let mut candidates: Vec<(ActivityId, f64)> =
            Vec::with_capacity(solution.problem().jobs().len());
        for _ in 0..context.num_jobs_to_remove {
            if solution.is_empty() {
                return;
            }

            candidates.clear();
            candidates.extend(solution.non_empty_routes_iter().flat_map(|route| {
                route
                    .activity_ids()
                    .iter()
                    .enumerate()
                    .map(|(index, &job_id)| {
                        let savings = compute_savings(solution.problem(), route, index);
                        (job_id, savings)
                    })
            }));

            candidates.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            let y: f64 = context.rng.random_range(0.0..1.0);
            let index = (y.powf(p) * candidates.len() as f64).floor() as usize;

            // Remove the activity with the worst savings
            if let Some(job_id) = candidates.get(index).map(|candidate| candidate.0) {
                solution.remove_service(job_id.into());
                solution.resync();
            } else {
                break;
            }
        }
    }
}
