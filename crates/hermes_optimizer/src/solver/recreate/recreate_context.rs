use std::hash::{Hash, Hasher};

use fxhash::FxHasher64;
use rand::{RngCore, rngs::SmallRng};

use crate::{
    problem::{job::JobIdx, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        constraints::{compute_insertion_score::compute_insertion_score, constraint::Constraint},
        insertion::Insertion,
        insertion_context::InsertionContext,
        noise::{JobNoiser, NoiseParams},
        recreate::recreate_strategy::RecreateStrategy,
        score::Score,
        solution::working_solution::WorkingSolution,
    },
};

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
    pub constraints: &'a Vec<Constraint>,
    pub problem: &'a VehicleRoutingProblem,
    pub noise_params: NoiseParams,
    pub thread_pool: &'a rayon::ThreadPool,
    pub insert_on_failure: bool,
}

impl<'a> RecreateContext<'a> {
    pub fn create_iteration_seed(&mut self) -> u64 {
        self.rng.next_u64()
    }

    pub fn create_noiser_seed(&self, iteration_seed: u64, job_id: JobIdx) -> u64 {
        let mut hasher = FxHasher64::default();

        // The job ID + a random number at each iteration should be enough for reproducibility
        (job_id.get(), iteration_seed).hash(&mut hasher);
        hasher.finish()
    }

    pub fn create_noiser(&self, seed: u64) -> JobNoiser {
        JobNoiser::new(seed, self.noise_params.clone())
    }

    pub fn compute_insertion_score(
        &self,
        solution: &WorkingSolution,
        insertion: &Insertion,
        best_score: Option<&Score>,
    ) -> Score {
        let context =
            InsertionContext::new(self.problem, solution, insertion, self.insert_on_failure);
        compute_insertion_score(self.constraints, &context, best_score)
    }

    pub fn should_insert(&self, score: &Score) -> bool {
        if self.insert_on_failure {
            true
        } else {
            !score.is_infeasible()
        }
    }

    pub fn insert_with_score_assertions(
        &self,
        solution: &mut WorkingSolution,
        insertion: Insertion,
        strategy: RecreateStrategy,
    ) {
        let cloned_solution = solution.clone();

        solution.insert(&insertion);

        if !self.insert_on_failure {
            let (_, current_score_analysis) =
                cloned_solution.compute_solution_score(self.constraints);
            let (score, analysis) = solution.compute_solution_score(self.constraints);
            if score.is_infeasible() {
                tracing::error!(
                    "({}) Insertion {:?} resulted in a score failure",
                    strategy,
                    insertion
                );
                tracing::error!("New score {:?}", analysis);

                tracing::error!("Current score {:?}", current_score_analysis);

                for constraint in self.constraints {
                    let score = constraint.compute_insertion_score(&InsertionContext::new(
                        self.problem,
                        &cloned_solution,
                        &insertion,
                        self.insert_on_failure,
                    ));

                    tracing::error!(
                        "Constraint {} obtained score {:?}",
                        constraint.constraint_name(),
                        score
                    )
                }

                insertion.route(&cloned_solution).dump(self.problem);
                // Dump new one
                insertion.route(solution).dump(self.problem);

                panic!("Bug: Insertion failure");
            }
        }
    }
}
