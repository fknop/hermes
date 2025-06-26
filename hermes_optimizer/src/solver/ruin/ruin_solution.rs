use crate::solver::working_solution::WorkingSolution;

use super::ruin_context::RuinContext;

pub trait RuinSolution {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext);
}
