use rand::RngCore;

use crate::solver::working_solution::WorkingSolution;

use super::ruin_context::RuinContext;

pub trait RuinSolution {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: RngCore;
}
