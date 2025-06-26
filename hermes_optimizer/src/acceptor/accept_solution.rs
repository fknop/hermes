use crate::solver::{
    accepted_solution::AcceptedSolution, score::Score, working_solution::WorkingSolution,
};

pub trait AcceptSolution {
    fn accept(
        &self,
        current_solutions: &[AcceptedSolution],
        solution: &WorkingSolution,
        score: &Score,
    ) -> bool;
}
