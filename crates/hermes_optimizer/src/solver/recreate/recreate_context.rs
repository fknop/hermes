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
        compute_insertion_score(
            self.constraints,
            &context,
            best_score,
            self.insert_on_failure,
        )
    }

    pub fn should_insert(&self, score: &Score) -> bool {
        if self.insert_on_failure {
            true
        } else {
            !score.is_failure()
        }
    }
}
