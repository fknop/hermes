use rand::rngs::SmallRng;

use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, solution::working_solution::WorkingSolution,
};

pub struct AcceptSolutionContext<'a> {
    pub iteration: usize,
    pub max_iterations: Option<usize>,
    pub max_solutions: usize,
    pub rng: &'a mut SmallRng,
}

pub trait AcceptSolution {
    fn accept(
        &self,
        current_solutions: &[AcceptedSolution],
        solution: &WorkingSolution,
        score: &Score,
        context: AcceptSolutionContext,
    ) -> bool;
}
