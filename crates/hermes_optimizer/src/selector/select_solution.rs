use crate::solver::{accepted_solution::AcceptedSolution, solution::population::Population};

pub trait SelectSolution {
    fn select_solution<'r>(
        &self,
        solutions: &'r Population,
        rng: &mut impl rand::Rng,
    ) -> Option<&'r AcceptedSolution>;
}
