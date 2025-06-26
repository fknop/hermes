use crate::solver::working_solution::WorkingSolution;

use super::recreate_context::RecreateContext;

pub trait RecreateSolution {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext);
}
