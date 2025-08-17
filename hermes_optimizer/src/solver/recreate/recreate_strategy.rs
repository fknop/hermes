use std::fmt::Display;

use serde::Serialize;

use crate::solver::working_solution::WorkingSolution;

use super::{
    best_insertion::{BestInsertion, BestInsertionSortMethod},
    recreate_context::RecreateContext,
    recreate_solution::RecreateSolution,
    regret_insertion::RegretInsertion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum RecreateStrategy {
    BestInsertion(BestInsertionSortMethod),
    RegretInsertion,
}

impl Display for RecreateStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BestInsertion(sort_method) => write!(f, "BestInsertion({sort_method})"),
            Self::RegretInsertion => write!(f, "RegretInsertion"),
        }
    }
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
