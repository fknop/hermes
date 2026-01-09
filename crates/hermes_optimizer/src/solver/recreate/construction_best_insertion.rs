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
                .map(|&job_id| {
                    let mut best_insertion_for_service: Option<Insertion> = None;
                    let mut best_score_for_service = Score::MAX;

                    for index in 0..solution.routes().len() {
                        let route_id = RouteIdx::new(index);
                        if let Some(entry) = insertion_cache.get(
                            route_id,
                            solution.route(route_id).version(),
                            job_id,
                        ) {
                            best_score_for_service = entry.score;
                            best_insertion_for_service = Some(entry.insertion.clone());
                        } else {
                            let mut best_score_for_route: Option<Score> = None;
                            let mut best_insertion_for_route: Option<Insertion> = None;
                            for_each_route_insertion(solution, route_id, job_id, |insertion| {
                                let score = context.compute_insertion_score(
                                    solution,
                                    &insertion,
                                    Some(&best_score_for_service),
                                );

                                if score < best_score_for_route.unwrap_or(Score::MAX) {
                                    best_score_for_route = Some(score);
                                    best_insertion_for_route = Some(insertion.clone());
                                }

                                if score < best_score_for_service {
                                    best_score_for_service = score;
                                    best_insertion_for_service = Some(insertion);
                                }
                            });

                            if let Some(best_insertion) = &best_insertion_for_route
                                && let Some(best_score) = best_score_for_route
                            {
                                insertion_cache.insert(
                                    route_id,
                                    solution.route(route_id).version(),
                                    job_id,
                                    best_score,
                                    best_insertion.clone(),
                                );
                            }
                        }
                    }

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

            if !context.should_insert(&best_score) {
                break;
            }

            if let Some(insertion) = best_insertion {
                solution.insert(&insertion);
            } else {
                panic!("No insertion possible")
            }
        }
    }
}

impl RecreateSolution for ConstructionBestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        ConstructionBestInsertion::insert_services(solution, context);
    }
}
