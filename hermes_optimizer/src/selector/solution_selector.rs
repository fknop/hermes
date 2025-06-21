use crate::solver::solution::Solution;

pub trait SolutionSelector {
    fn select_solution<'a>(&self, solutions: &'a [Solution]) -> Option<&'a Solution>;
}
