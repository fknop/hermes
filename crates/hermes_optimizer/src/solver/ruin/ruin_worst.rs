use std::f64;

use crate::{
    problem::{
        job::{ActivityId, JobIdx},
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::solution::{
        route::WorkingSolutionRoute,
        route_id::{self, RouteIdx},
        working_solution::WorkingSolution,
    },
    utils::enumerate_idx::EnumerateIdx,
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments: right now it only compute savings from activity independently
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
        Some(
            route
                .activity(index - 1)
                .job_activity(problem)
                .location_id(),
        )
    };

    let next_location_id = if index == route.activity_ids().len() - 1 {
        if vehicle.should_return_to_depot() {
            vehicle.depot_location_id()
        } else {
            None
        }
    } else {
        Some(
            route
                .activity(index + 1)
                .job_activity(problem)
                .location_id(),
        )
    };

    let location_id = activity.job_activity(problem).location_id();

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

        let mut candidates: Vec<Savings> =
            Vec::with_capacity(solution.problem().jobs().len() - solution.unassigned_jobs().len());

        let mut route_ids = solution
            .routes()
            .iter()
            .enumerate_idx()
            .filter(|(_, route): &(RouteIdx, &WorkingSolutionRoute)| !route.is_empty())
            .map(|(id, _): (RouteIdx, &WorkingSolutionRoute)| id)
            .collect::<Vec<RouteIdx>>();

        for _ in 0..context.num_jobs_to_remove {
            if solution.is_empty() {
                return;
            }

            // Instead of recomputing every candidates from a route every loop
            // We could only recompute the neighbors of the activities that were removed
            candidates.extend(route_ids.iter().flat_map(|route_id| {
                let route = solution.route(*route_id);
                route
                    .activity_ids()
                    .iter()
                    .enumerate()
                    .map(|(index, &activity_id)| {
                        let savings = compute_savings(solution.problem(), route, index);
                        Savings {
                            job_id: activity_id.job_id(),
                            route_id: *route_id,
                            savings,
                        }
                    })
            }));

            candidates.sort_unstable_by(|a, b| b.savings.partial_cmp(&a.savings).unwrap());

            let y: f64 = context.rng.random_range(0.0..1.0);
            let index = (y.powf(p) * candidates.len() as f64).floor() as usize;

            // Remove the activity with the worst savings
            if let Some(candidate) = candidates.get(index) {
                // TODO: check for shipments
                let removed = solution.remove_job(candidate.job_id);

                if removed {
                    route_ids.clear();

                    // Only route_id changed, no need to recompute savings for other routes
                    route_ids.push(candidate.route_id);
                }
            } else {
                break;
            }

            // Remove candidates that are in route_ids
            candidates.retain(|candidate| !route_ids.contains(&candidate.route_id));
        }
    }
}

struct Savings {
    route_id: RouteIdx,
    job_id: JobIdx,
    savings: f64,
}
