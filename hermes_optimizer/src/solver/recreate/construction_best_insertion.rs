use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::solver::{
    insertion::{Insertion, for_each_insertion},
    score::Score,
    solution::working_solution::WorkingSolution,
};

use super::{recreate_context::RecreateContext, recreate_solution::RecreateSolution};

#[derive(Default)]
pub struct ConstructionBestInsertion;

impl ConstructionBestInsertion {
    pub fn insert_services(solution: &mut WorkingSolution, context: RecreateContext) {
        context.thread_pool.install(|| {
            while !solution.unassigned_jobs().is_empty() {
                let mut best_insertion: Option<Insertion> = None;
                let mut best_score = Score::MAX;

                let results = solution
                    .unassigned_jobs()
                    .par_iter()
                    .map(|&job_id| {
                        let mut best_insertion_for_service: Option<Insertion> = None;
                        let mut best_score_for_service = Score::MAX;

                        for_each_insertion(solution, job_id, |insertion| {
                            let score = context.compute_insertion_score(
                                solution,
                                &insertion,
                                Some(&best_score_for_service),
                            );

                            if score < best_score_for_service {
                                best_score_for_service = score;
                                best_insertion_for_service = Some(insertion);
                            }
                        });

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
        });
    }
}

impl RecreateSolution for ConstructionBestInsertion {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        ConstructionBestInsertion::insert_services(solution, context);
    }
}
