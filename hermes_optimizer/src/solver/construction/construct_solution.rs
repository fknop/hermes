use std::sync::Arc;

use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::constraint::Constraint,
        recreate::{best_insertion::BestInsertion, recreate_context::RecreateContext},
        working_solution::WorkingSolution,
    },
};

pub fn construct_solution(
    problem: &Arc<VehicleRoutingProblem>,
    rng: &mut SmallRng,
    constraints: &Vec<Constraint>,
) -> WorkingSolution {
    let mut solution = WorkingSolution::new(Arc::clone(problem));
    let mut services: Vec<_> = (0..problem.services().len()).collect();

    let vehicles = problem.vehicles();
    let depot_location = vehicles
        .iter()
        .filter_map(|vehicle| vehicle.depot_location_id())
        .map(|location_id| problem.location(location_id))
        .next();

    if let Some(depot_location) = depot_location {
        services.sort_by(|&first, &second| {
            let first = problem.service_location(first);
            let second = problem.service_location(second);

            depot_location
                .bearing(first)
                .partial_cmp(&depot_location.bearing(second))
                .unwrap()
        });
    }

    BestInsertion::insert_services(
        &services,
        &mut solution,
        RecreateContext { rng, constraints },
    );

    solution
}
