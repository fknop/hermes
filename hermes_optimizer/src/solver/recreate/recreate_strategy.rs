use crate::solver::working_solution::WorkingSolution;

use super::{
    best_insertion::{BestInsertion, BestInsertionSortMethod},
    recreate_context::RecreateContext,
    recreate_solution::RecreateSolution,
    regret_insertion::RegretInsertion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecreateStrategy {
    BestInsertion(BestInsertionSortMethod),
    RegretInsertion,
}

impl RecreateSolution for RecreateStrategy {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        match self {
            RecreateStrategy::BestInsertion(sort_method) => {
                let strategy = BestInsertion::new(*sort_method);
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::RegretInsertion => {
                let strategy = RegretInsertion::new(3);
                strategy.recreate_solution(solution, context);
            }
        }
    }
}
