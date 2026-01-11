use crate::solver::{
    insertion::{Insertion, for_each_route_insertion},
    insertion_cache::InsertionCache,
    score::Score,
    solution::{route_id::RouteIdx, working_solution::WorkingSolution},
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct ConstructionBestInsertion;

impl ConstructionBestInsertion {
    pub fn insert_services(solution: &mut WorkingSolution, context: RecreateContext) {
        // Insertions into routes will keep the same cost accross insertions while the routes remain unchanged
        let mut insertion_cache = InsertionCache::new();

        while !solution.unassigned_jobs().is_empty() {
            let mut best_insertion: Option<Insertion> = None;
            let mut best_score = Score::MAX;

            let results = solution
                .unassigned_jobs()
                .iter()
                .filter_map(|&job_id| {
                    let mut best_insertion_for_service: Option<Insertion> = None;
                    let mut best_score_for_service: Option<Score> = None;

                    for index in 0..solution.routes().len() {
                        let route_id = RouteIdx::new(index);
                        let version = solution.route(route_id).version();

                        if let Some(entry) = insertion_cache.get(route_id, version, job_id) {
                            if entry.score < best_score_for_service.unwrap_or(Score::MAX) {
                                best_score_for_service = Some(entry.score);
                                best_insertion_for_service = Some(entry.insertion.clone());
                            }
                        } else {
                            let mut best_score_for_route: Option<Score> = None;
                            let mut best_insertion_for_route: Option<Insertion> = None;
                            for_each_route_insertion(solution, route_id, job_id, |insertion| {
                                let score = context.compute_insertion_score(
                                    solution,
                                    &insertion,
                                    best_score_for_service.as_ref(),
                                );

                                if score < best_score_for_route.unwrap_or(Score::MAX) {
                                    best_score_for_route = Some(score);
                                    best_insertion_for_route = Some(insertion.clone());
                                }

                                if score < best_score_for_service.unwrap_or(Score::MAX) {
                                    best_score_for_service = Some(score);
                                    best_insertion_for_service = Some(insertion);
                                }
                            });

                            if let Some(best_insertion) = &best_insertion_for_route
                                && let Some(best_score) = best_score_for_route
                            {
                                insertion_cache.insert(
                                    route_id,
                                    version,
                                    job_id,
                                    best_score,
                                    best_insertion.clone(),
                                );
                            }
                        }
                    }

                    match (best_insertion_for_service, best_score_for_service) {
                        (Some(insertion), Some(score)) => Some((insertion, score)),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>();

            for result in results {
                let (insertion, score) = result;
                if score < best_score && context.should_insert(&score) {
                    best_score = score;
                    best_insertion = Some(insertion);
                }
            }

            if let Some(insertion) = best_insertion {
                solution.insert(&insertion);
            } else {
                break;
                // panic!("No insertion possible")
            }
        }
    }
}

impl RecreateSolution for ConstructionBestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        ConstructionBestInsertion::insert_services(solution, context);
    }
}
