use crate::solution::Solution;

pub trait SolutionSelector {
    fn accept_solution(solutions: &[Solution]) -> Option<&Solution>;
}
