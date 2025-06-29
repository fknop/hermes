use rand::Rng;

use crate::solver::{
    constraints::compute_constraints_score::compute_insertion_score,
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    score::Score,
    working_solution::{WorkingSolution, compute_insertion_context},
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

pub struct BestInsertion;

impl RecreateSolution for BestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        while let Some(&service_id) = solution.unassigned_services().iter().next() {
            let mut best_insertion: Option<Insertion> = None;
            let mut best_score = Score::MAX;

            let routes = solution.routes();
            for (route_id, route) in routes.iter().enumerate() {
                for position in 0..route.activities().len() {
                    let insertion = Insertion::ExistingRoute(ExistingRouteInsertion {
                        route_id,
                        service_id,
                        position,
                    });

                    let score = context.compute_insertion_score(solution, &insertion);

                    if score < best_score {
                        best_score = score;
                        best_insertion = Some(insertion);
                    }
                }
            }

            if solution.has_available_vehicle() {
                for vehicle_id in solution.available_vehicles() {
                    let new_route_insertion = Insertion::NewRoute(NewRouteInsertion {
                        service_id,
                        vehicle_id,
                    });

                    let score = context.compute_insertion_score(solution, &new_route_insertion);

                    if score < best_score {
                        // best_score = score;
                        best_insertion = Some(new_route_insertion);
                    }
                }
            }

            if let Some(insertion) = best_insertion {
                solution.insert_service(&insertion);
            } else {
                panic!("No insertion possible")
            }
        }
    }
}
