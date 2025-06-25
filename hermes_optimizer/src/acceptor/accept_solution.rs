use crate::solver::{score::Score, solution::Solution, working_solution::WorkingSolution};

pub trait AcceptSolution {
    fn accept(
        &self,
        current_solutions: &[Solution],
        solution: &WorkingSolution,
        score: &Score,
    ) -> bool;
}
