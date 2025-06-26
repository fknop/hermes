use crate::solver::working_solution::WorkingSolution;

use super::{
    random_insertion::RandomInsertion, recreate_context::RecreateContext,
    recreate_solution::RecreateSolution,
};

#[derive(Clone, Copy, Debug)]
pub enum RecreateStrategy {
    RandomInsertion,
    BestInsertion,
}

impl RecreateSolution for RecreateStrategy {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        match self {
            RecreateStrategy::RandomInsertion => {
                let strategy = RandomInsertion;
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::BestInsertion => {
                todo!()
            }
        }
    }
}
