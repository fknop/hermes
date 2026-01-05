use rayon::iter::ParallelIterator;

use crate::solver::{
    insertion::{Insertion, for_each_route_insertion},
    score::Score,
    solution::{route_id::RouteIdx, working_solution::WorkingSolution},
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct ConstructionBestInsertion;

impl ConstructionBestInsertion {
    pub fn insert_services(solution: &mut WorkingSolution, context: RecreateContext) {
        context.thread_pool.install(|| {
            // Insertions into routes will keep the same cost accross insertions while the routes remain unchanged
            let mut best_routes_cache: Vec<Vec<(Score, Option<Insertion>)>> =
                vec![
                    vec![(Score::MAX, None); solution.problem().jobs().len()];
                    solution.routes().len()
                ];

            while !solution.unassigned_jobs().is_empty() {
                let mut best_insertion: Option<Insertion> = None;
                let mut best_score = Score::MAX;

                if solution.routes().len() > best_routes_cache.len() {
                    best_routes_cache.resize(
                        solution.routes().len(),
                        vec![(Score::MAX, None); solution.problem().jobs().len()],
                    );
                }

                let results = solution
                    .unassigned_jobs()
                    .iter()
                    .map(|&job_id| {
                        let mut best_insertion_for_service: Option<Insertion> = None;
                        let mut best_score_for_service = Score::MAX;

                        for route_id in 0..best_routes_cache.len() {
                            let (job_score, _) = best_routes_cache[route_id][job_id.get()];
                            if job_score == Score::MAX {
                                let mut best_score_for_route = Score::MAX;
                                let mut best_insertion_for_route: Option<Insertion> = None;
                                for_each_route_insertion(
                                    solution,
                                    RouteIdx::new(route_id),
                                    job_id,
                                    |insertion| {
                                        let score = context.compute_insertion_score(
                                            solution,
                                            &insertion,
                                            Some(&best_score_for_service),
                                        );

                                        if score < best_score_for_route {
                                            best_score_for_route = score;
                                            best_insertion_for_route = Some(insertion.clone());
                                        }

                                        if score < best_score_for_service {
                                            best_score_for_service = score;
                                            best_insertion_for_service = Some(insertion);
                                        }
                                    },
                                );

                                best_routes_cache[route_id][job_id.get()] =
                                    (best_score_for_route, best_insertion_for_route);
                            } else if job_score < best_score_for_service {
                                best_score_for_service = job_score;
                                best_insertion_for_service =
                                    best_routes_cache[route_id][job_id.get()].1.clone();
                            }
                        }

                        // for_each_insertion(solution, job_id, |insertion| {
                        //     let score = context.compute_insertion_score(
                        //         solution,
                        //         &insertion,
                        //         Some(&best_score_for_service),
                        //     );

                        //     if score < best_score_for_service {
                        //         best_score_for_service = score;
                        //         best_insertion_for_service = Some(insertion);
                        //     }
                        // });

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
                    best_routes_cache[insertion.route_id().get()].fill_with(|| (Score::MAX, None));
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
