use rand::rngs::SmallRng;

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        constraints::{compute_insertion_score::compute_insertion_score, constraint::Constraint},
        insertion::Insertion,
        noise::NoiseGenerator,
        score::Score,
        working_solution::{WorkingSolution, compute_insertion_context},
    },
};

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
    pub constraints: &'a Vec<Constraint>,
    pub problem: &'a VehicleRoutingProblem,
    pub noise_generator: &'a NoiseGenerator,
}

impl<'a> RecreateContext<'a> {
    pub fn compute_insertion_score(
        &mut self,
        solution: &WorkingSolution,
        insertion: &Insertion,
    ) -> Score {
        let context = compute_insertion_context(self.problem, solution, insertion);
        compute_insertion_score(self.constraints, &context, self.noise_generator, self.rng)
    }
}
