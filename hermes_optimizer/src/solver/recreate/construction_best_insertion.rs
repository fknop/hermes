use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::solver::{
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    score::Score,
    solution::working_solution::WorkingSolution,
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct ConstructionBestInsertion;

impl ConstructionBestInsertion {
    pub fn insert_services(solution: &mut WorkingSolution, context: RecreateContext) {
        context.thread_pool.install(|| {
            while !solution.unassigned_services().is_empty() {
                let mut best_insertion: Option<Insertion> = None;
                let mut best_score = Score::MAX;

                let results = solution
                    .unassigned_services()
                    .par_iter()
                    .map(|&service_id| {
                        let mut best_insertion_for_service: Option<Insertion> = None;
                        let mut best_score_for_service = Score::MAX;
                        let routes = solution.routes();

                        for (route_id, route) in routes.iter().enumerate() {
                            for position in 0..=route.activities().len() {
                                let insertion = if route.is_empty() {
                                    Insertion::NewRoute(NewRouteInsertion {
                                        service_id,
                                        vehicle_id: route.vehicle_id(),
                                    })
                                } else {
                                    Insertion::ExistingRoute(ExistingRouteInsertion {
                                        route_id,
                                        service_id,
                                        position,
                                    })
                                };

                                let score = context.compute_insertion_score(solution, &insertion);

                                if score < best_score_for_service {
                                    best_score_for_service = score;
                                    best_insertion_for_service = Some(insertion);
                                }
                            }
                        }

                        // if solution.has_available_vehicle() {
                        //     for vehicle_id in solution.available_vehicles_iter() {
                        //         let new_route_insertion = Insertion::NewRoute(NewRouteInsertion {
                        //             service_id,
                        //             vehicle_id,
                        //         });

                        //         let score =
                        //             context.compute_insertion_score(solution, &new_route_insertion);

                        //         if score < best_score_for_service {
                        //             best_score_for_service = score;
                        //             best_insertion_for_service = Some(new_route_insertion);
                        //         }
                        //     }
                        // }

                        (best_insertion_for_service, best_score_for_service)
                    })
                    .collect::<Vec<_>>();

                for result in results {
                    if let (Some(insertion), score) = result
                        && score < best_score
                    {
                        best_score = score;
                        best_insertion = Some(insertion);
                    }
                }

                // for &service_id in solution.unassigned_services().iter() {
                //     let routes = solution.routes();

                //     for (route_id, route) in routes.iter().enumerate() {
                //         for position in 0..=route.activities().len() {
                //             let insertion = Insertion::ExistingRoute(ExistingRouteInsertion {
                //                 route_id,
                //                 service_id,
                //                 position,
                //             });

                //             let score = context.compute_insertion_score(solution, &insertion);

                //             if score < best_score {
                //                 best_score = score;
                //                 best_insertion = Some(insertion);
                //             }
                //         }
                //     }

                //     if solution.has_available_vehicle() {
                //         for vehicle_id in solution.available_vehicles_iter() {
                //             let new_route_insertion = Insertion::NewRoute(NewRouteInsertion {
                //                 service_id,
                //                 vehicle_id,
                //             });

                //             let score = context.compute_insertion_score(solution, &new_route_insertion);

                //             if score < best_score {
                //                 // best_score = score;
                //                 best_insertion = Some(new_route_insertion);
                //             }
                //         }
                //     }
                // }

                if let Some(insertion) = best_insertion {
                    solution.insert_service(&insertion);
                } else {
                    panic!("No insertion possible")
                }
            }
        });
    }
}

impl RecreateSolution for ConstructionBestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        ConstructionBestInsertion::insert_services(solution, context);
    }
}
