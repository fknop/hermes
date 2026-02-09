use crate::solver::accepted_solution::AcceptedSolution;

pub trait SelectSolution {
    fn select_solution<'r>(
        &self,
        solutions: &'r [AcceptedSolution],
        rng: &mut impl rand::Rng,
    ) -> Option<&'r AcceptedSolution>;
}
