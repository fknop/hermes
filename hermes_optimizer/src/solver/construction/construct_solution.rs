use std::sync::Arc;

use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::constraint::Constraint,
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        noise::NoiseGenerator,
        recreate::{
            best_insertion::BestInsertion, recreate_context::RecreateContext,
            regret_insertion::RegretInsertion,
        },
        working_solution::WorkingSolution,
    },
};

pub fn construct_solution(
    problem: &Arc<VehicleRoutingProblem>,
    rng: &mut SmallRng,
    constraints: &Vec<Constraint>,
    noise_generator: &NoiseGenerator,
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

    // services.sort_by_key(|&service| {
    //     if let Some(depot_id) = depot_id {
    //         let depot_location = problem.location(depot_id);
    //         let service_location = problem.service_location(service);
    //         let angle = depot_location.bearing(service_location);

    //         (angle * 1000.0).round() as i64
    //     } else {
    //         0 // Fallback if no depot is found
    //     }
    // });

    BestInsertion::insert_services(
        &services,
        &mut solution,
        RecreateContext {
            rng,
            constraints,
            noise_generator,
        },
    );

    // let regret = RegretInsertion::new(3);
    // regret.insert_services(
    //     &mut solution,
    //     RecreateContext {
    //         rng,
    //         constraints,
    //         noise_generator,
    //     },
    // );

    solution
}
