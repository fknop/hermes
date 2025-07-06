use rand::{Rng, seq::SliceRandom};

use crate::{
    problem::service::ServiceId,
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        score::Score,
        working_solution::WorkingSolution,
    },
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

pub struct BestInsertion;

impl BestInsertion {
    pub fn insert_services(
        unassigned_services: &Vec<ServiceId>,
        solution: &mut WorkingSolution,
        context: RecreateContext,
    ) {
        for &service_id in unassigned_services {
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

impl RecreateSolution for BestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        let mut unassigned_services: Vec<_> =
            solution.unassigned_services().iter().copied().collect();
        unassigned_services.shuffle(context.rng);

        BestInsertion::insert_services(&unassigned_services, solution, context);
    }
}
