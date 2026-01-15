use std::fmt::Display;

use serde::Serialize;

use crate::solver::{
    recreate::best_insertion::BestInsertionSortStrategy,
    solution::working_solution::WorkingSolution,
};

use super::{
    best_insertion::{BestInsertion, BestInsertionParams},
    construction_best_insertion::ConstructionBestInsertion,
    recreate_context::RecreateContext,
    recreate_solution::RecreateSolution,
    regret_insertion::RegretInsertion,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RecreateStrategy {
    CompleteBestInsertion,
    BestInsertion(BestInsertionSortStrategy),
    RegretInsertion(usize),
}

impl Serialize for RecreateStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Display for RecreateStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompleteBestInsertion => write!(f, "CompleteBestInsertion"),
            Self::BestInsertion(sort_method) => write!(f, "BestInsertion({sort_method})"),
            Self::RegretInsertion(k) => write!(f, "RegretInsertion({k})"),
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
                    sort_strategy: *sort_method,
                    blink_rate: 0.01,
                });
                strategy.recreate_solution(solution, context);
            }
            RecreateStrategy::RegretInsertion(k) => {
                let strategy = RegretInsertion::new(*k);
                strategy.recreate_solution(solution, context);
            }
        }

        // solution.resync();
    }
}
