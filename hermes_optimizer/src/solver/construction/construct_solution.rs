use std::sync::Arc;

use rand::rngs::SmallRng;

use crate::{
    problem::{travel_cost_matrix::Distance, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        constraints::constraint::Constraint,
        noise::NoiseGenerator,
        recreate::{
            best_insertion::BestInsertion,
            recreate_context::RecreateContext,
            recreate_solution::RecreateSolution,
            regret_insertion::{self, RegretInsertion},
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

    services.sort_by_key(|&service| {
        let distance_from_depot = problem
            .vehicles()
            .iter()
            .filter_map(|vehicle| vehicle.depot_location_id())
            .map(|depot_location_id| {
                problem.travel_distance(depot_location_id, problem.service_location(service).id())
            })
            .sum::<Distance>()
            / problem.vehicles().len() as Distance;

        distance_from_depot.round() as i64
    });

    BestInsertion::insert_services(
        &services,
        &mut solution,
        RecreateContext {
            rng,
            constraints,
            noise_generator,
        },
    );

    solution
}
