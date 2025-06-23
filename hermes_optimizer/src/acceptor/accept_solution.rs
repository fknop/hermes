use crate::solver::solution::Solution;

pub trait AcceptSolution {
    fn accept_solution(&self, current_solutions: &[Solution], solution: &Solution) -> bool;
}
