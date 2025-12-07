use rand::Rng;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::solver::{
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    score::Score,
    solution::working_solution::WorkingSolution,
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

/// Implements a Regret-k Insertion heuristic.
///
/// In each step of the construction process, this heuristic decides which service to insert next.
/// Instead of picking the overall cheapest insertion (like BestInsertion), it prioritizes
/// services that are "hardest to place".
///
/// The "hardness" is quantified by a "regret" value. For each unassigned service, we find its `k`
/// best possible insertion positions. The regret is the cost difference between these
/// better options and the single best option.
///
/// Regret = sum_{i=2 to k} (cost_of_i_th_best_insertion - cost_of_best_insertion)
///
/// The service with the highest regret is chosen to be inserted at its best position.
/// This helps to avoid situations where placing easy services first makes it very
/// expensive or impossible to place other services later.
pub struct RegretInsertion {
    /// The 'k' in Regret-k. Determines how many best insertion options are considered
    /// for calculating the regret value. A common value is 2 or 3.
    pub k: usize,
}

impl RegretInsertion {
    /// Creates a new RegretInsertion heuristic.
    ///
    /// # Panics
    /// Panics if k < 2, as regret calculation requires at least two options to compare.
    pub fn new(k: usize) -> Self {
        assert!(k >= 2, "Regret-k heuristic requires k to be at least 2.");
        Self { k }
    }

    fn compute_best_insertion(
        &self,
        solution: &mut WorkingSolution,
        context: &mut RecreateContext,
    ) -> Option<Insertion> {
        let regret_values: Vec<(Score, Insertion)> = solution
            .unassigned_services()
            .par_iter()
            .filter_map(|&service_id| {
                let mut potential_insertions: Vec<(Score, Insertion)> = Vec::with_capacity(
                    // One insertion after each activity
                    (context.problem.jobs().len() - solution.unassigned_services().len())
                    // One insertion at the start of every route
                    + solution.routes().len(),
                );

                // Find all possible insertions in existing routes
                for (route_id, route) in solution.routes().iter().enumerate() {
                    // We can insert at any position, including the end
                    for position in 0..=route.activity_ids().len() {
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
                        let score = context.compute_insertion_score(solution, &insertion, None);

                        // Only consider valid insertions
                        potential_insertions.push((score, insertion));
                    }
                }

                // Consider creating a new route if a vehicle is available
                // if solution.has_available_vehicle() {
                //     for vehicle_id in solution.available_vehicles_iter() {
                //         let insertion = Insertion::NewRoute(NewRouteInsertion {
                //             service_id,
                //             vehicle_id,
                //         });
                //         let score = context.compute_insertion_score(solution, &insertion);
                //         potential_insertions.push((score, insertion));
                //     }
                // }

                // If no valid insertion was found for this service, skip it
                if potential_insertions.is_empty() {
                    return None;
                }

                // Sort insertions by score to find the best ones
                potential_insertions.sort_unstable_by(|a, b| a.0.cmp(&b.0));

                // 2. Calculate the regret value for this service
                let best_insertion = &potential_insertions[0];
                let best_score = best_insertion.0;
                let mut regret_value = Score::zero();

                // The number of insertions to consider for the regret sum
                let limit = self.k.min(potential_insertions.len());

                // Regret = sum of differences between k-th best and the best
                for potential_insertion in potential_insertions.iter().skip(1).take(limit) {
                    regret_value += potential_insertion.0 - best_score;
                }

                Some((regret_value, best_insertion.1.clone()))
            })
            .collect();

        let mut best_insertion_for_max_regret: Option<Insertion> = None;
        let mut max_regret = Score::MIN;

        for (regret_value, insertion) in regret_values {
            if regret_value > max_regret
                || (regret_value == max_regret && context.rng.random_bool(0.5))
            {
                max_regret = regret_value;
                best_insertion_for_max_regret = Some(insertion);
            }
        }

        best_insertion_for_max_regret
    }

    pub fn insert_services(&self, solution: &mut WorkingSolution, mut context: RecreateContext) {
        // Create vectors with predefined capacities to avoid reallocations in each loop
        // let mut unassigned_services: Vec<ServiceId> =
        //     Vec::with_capacity(solution.unassigned_services().len());
        // let mut potential_insertions: Vec<(Score, Insertion)> = Vec::with_capacity(
        //     // One insertion after each activity
        //     (context.problem.services().len() - solution.unassigned_services().len())
        //         // One insertion at the start of every route
        //         + solution.routes().len()
        //         // One insertion per available vehicle
        //         + solution.num_available_vehicles(),
        // );

        while !solution.unassigned_services().is_empty() {
            // let mut best_insertion_for_max_regret: Option<Insertion> = None;
            // let mut max_regret = Score::MIN;

            // unassigned_services.clear();
            // Take a snapshot of unassigned services for this iteration
            // unassigned_services.extend(solution.unassigned_services().iter());

            let best_insertion_for_max_regret = context
                .thread_pool
                .install(|| self.compute_best_insertion(solution, &mut context));

            // // 1. Calculate regret for EACH unassigned service
            // for &service_id in solution.unassigned_services().iter() {
            //     potential_insertions.clear();

            //     // Find all possible insertions in existing routes
            //     for (route_id, route) in solution.routes().iter().enumerate() {
            //         // We can insert at any position, including the end
            //         for position in 0..=route.activities().len() {
            //             let insertion = Insertion::ExistingRoute(ExistingRouteInsertion {
            //                 route_id,
            //                 service_id,
            //                 position,
            //             });
            //             let score = context.compute_insertion_score(solution, &insertion);

            //             // Only consider valid insertions
            //             potential_insertions.push((score, insertion));
            //         }
            //     }

            //     // Consider creating a new route if a vehicle is available
            //     if solution.has_available_vehicle() {
            //         for vehicle_id in solution.available_vehicles_iter() {
            //             let insertion = Insertion::NewRoute(NewRouteInsertion {
            //                 service_id,
            //                 vehicle_id,
            //             });
            //             let score = context.compute_insertion_score(solution, &insertion);
            //             potential_insertions.push((score, insertion));
            //         }
            //     }

            //     // If no valid insertion was found for this service, skip it
            //     if potential_insertions.is_empty() {
            //         continue;
            //     }

            //     // Sort insertions by score to find the best ones
            //     potential_insertions.sort_by(|a, b| a.0.cmp(&b.0));

            //     // 2. Calculate the regret value for this service
            //     let best_insertion = &potential_insertions[0];
            //     let best_score = best_insertion.0;
            //     let mut regret_value = Score::zero();

            //     // The number of insertions to consider for the regret sum
            //     let limit = self.k.min(potential_insertions.len());

            //     // Regret = sum of differences between k-th best and the best
            //     for potential_insertion in potential_insertions.iter().skip(1).take(limit) {
            //         regret_value += potential_insertion.0 - best_score;
            //     }

            //     // 3. Check if this service has the highest regret so far
            //     // We use a random tie-breaker to introduce diversity.
            //     if regret_value > max_regret
            //         || (regret_value == max_regret && context.rng.random_bool(0.5))
            //     {
            //         max_regret = regret_value;
            //         best_insertion_for_max_regret = Some(best_insertion.1.clone());
            //     }
            // }

            // 4. Perform the insertion of the service with the highest regret
            if let Some(insertion) = best_insertion_for_max_regret {
                solution.insert_service(&insertion);
            } else {
                panic!("no insertion possible");
                // If no service could be inserted (e.g., all remaining are infeasible),
                // we stop the insertion process. The rest will remain unassigned.
                // break;
            }
        }
    }
}

impl RecreateSolution for RegretInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        self.insert_services(solution, context);
    }
}
