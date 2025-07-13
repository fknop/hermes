use crate::solver::accepted_solution::AcceptedSolution;

pub trait SelectSolution {
    fn select_solution<'r, 'a>(
        &self,
        solutions: &'r [AcceptedSolution],
    ) -> Option<&'r AcceptedSolution>;
}
