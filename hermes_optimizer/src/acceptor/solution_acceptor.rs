use crate::solver::solution::Solution;

pub trait SolutionAcceptor {
    fn accept_solution(solution: &Solution) -> bool;
}
