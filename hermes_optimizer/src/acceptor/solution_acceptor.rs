use crate::solution::Solution;

pub trait SolutionAcceptor {
    fn accept_solution(solution: &Solution) -> bool;
}
