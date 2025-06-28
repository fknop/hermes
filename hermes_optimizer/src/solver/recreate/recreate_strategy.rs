use crate::solver::working_solution::WorkingSolution;

use super::{
    best_insertion::BestInsertion, random_insertion::RandomInsertion,
    recreate_context::RecreateContext, recreate_solution::RecreateSolution,
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
                let strategy = BestInsertion;
                strategy.recreate_solution(solution, context);
            }
        }
    }
}
