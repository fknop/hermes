use crate::solver::working_solution::WorkingSolution;

use super::{
    best_insertion::BestInsertion, recreate_context::RecreateContext,
    recreate_solution::RecreateSolution, regret_insertion::RegretInsertion,
};

#[derive(Clone, Copy, Debug)]
pub enum RecreateStrategy {
    BestInsertion,
    RegretInsertion,
}

impl RecreateSolution for RecreateStrategy {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        match self {
            RecreateStrategy::BestInsertion => {
                let strategy = BestInsertion;
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::RegretInsertion => {
                let strategy = RegretInsertion::new(2);
                strategy.recreate_solution(solution, context);
            }
        }
    }
}
