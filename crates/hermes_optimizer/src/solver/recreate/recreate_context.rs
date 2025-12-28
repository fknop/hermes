use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::{compute_insertion_score::compute_insertion_score, constraint::Constraint},
        insertion::Insertion,
        insertion_context::InsertionContext,
        noise::NoiseGenerator,
        score::Score,
        solution::working_solution::WorkingSolution,
    },
};

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
    pub constraints: &'a Vec<Constraint>,
    pub problem: &'a VehicleRoutingProblem,
    pub noise_generator: Option<&'a NoiseGenerator>,
    pub thread_pool: &'a rayon::ThreadPool,
    pub insert_on_failure: bool,
}

impl<'a> RecreateContext<'a> {
    pub fn compute_insertion_score(
        &self,
        solution: &WorkingSolution,
        insertion: &Insertion,
        best_score: Option<&Score>,
    ) -> Score {
        let context = InsertionContext::new(self.problem, solution, insertion);
        compute_insertion_score(self.constraints, &context, best_score)
            + self.noise_generator.map_or(Score::ZERO, |noise_generator| {
                Score::soft(noise_generator.create_noise(context.insertion.job_idx()))
            })
    }

    pub fn should_insert(&self, score: &Score) -> bool {
        if self.insert_on_failure {
            true
        } else {
            !score.is_failure()
        }
    }
}
