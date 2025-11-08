use std::fmt::Display;

use serde::Serialize;

use crate::solver::working_solution::WorkingSolution;

use super::{
    best_insertion::{BestInsertion, BestInsertionParams, BestInsertionSortMethod},
    construction_best_insertion::ConstructionBestInsertion,
    recreate_context::RecreateContext,
    recreate_solution::RecreateSolution,
    regret_insertion::RegretInsertion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum RecreateStrategy {
    CompleteBestInsertion,
    BestInsertion(BestInsertionSortMethod),
    RegretInsertion,
}

impl Display for RecreateStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompleteBestInsertion => write!(f, "CompleteBestInsertion"),
            Self::BestInsertion(sort_method) => write!(f, "BestInsertion({sort_method})"),
            Self::RegretInsertion => write!(f, "RegretInsertion"),
        }
    }
}

impl RecreateSolution for RecreateStrategy {
    fn recreate_solution(&self, solution: &mut WorkingSolution, context: RecreateContext) {
        match self {
            RecreateStrategy::CompleteBestInsertion => {
                let strategy = ConstructionBestInsertion;
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::BestInsertion(sort_method) => {
                let strategy = BestInsertion::new(BestInsertionParams {
                    sort_method: *sort_method,
                    blink_rate: 0.0,
                });
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::RegretInsertion => {
                let strategy = RegretInsertion::new(2);
                strategy.recreate_solution(solution, context);
            }
        }
    }
}
