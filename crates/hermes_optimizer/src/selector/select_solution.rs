use rand::rngs::SmallRng;

use crate::solver::accepted_solution::AcceptedSolution;

pub trait SelectSolution {
    fn select_solution<'r>(
        &self,
        solutions: &'r [AcceptedSolution],
        rng: &mut SmallRng,
    ) -> Option<&'r AcceptedSolution>;
}
