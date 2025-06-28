use rand::Rng;
use tracing::info;

use crate::solver::{
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    working_solution::WorkingSolution,
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

pub struct RandomInsertion;

impl RecreateSolution for RandomInsertion {
    fn recreate_solution(
        &self,
        solution: &mut WorkingSolution,
        RecreateContext { rng, .. }: RecreateContext,
    ) {
        while let Some(&service_id) = solution.unassigned_services().iter().next() {
            let num_routes = solution.routes().len();

            let create_new_route = if solution.has_available_vehicle() {
                rng.random_ratio(1, num_routes as u32 + 1)
            } else {
                false
            };

            if create_new_route {
                solution.insert_service(&Insertion::NewRoute(NewRouteInsertion {
                    service_id,
                    vehicle_id: solution.available_vehicle().unwrap(),
                }));
            } else {
                let route_id = rng.random_range(0..num_routes);

                solution.insert_service(&Insertion::ExistingRoute(ExistingRouteInsertion {
                    route_id,
                    service_id,
                    position: rng.random_range(0..solution.route(route_id).activities().len()),
                }));
            }
        }
    }
}
