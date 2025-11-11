use crate::solver::{
    insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
    working_solution::WorkingSolution,
};

use super::transport_cost_constraint::TransportCostConstraint;

pub trait GlobalConstraint {
    fn score_level(&self) -> ScoreLevel;
    fn compute_score(&self, solution: &WorkingSolution) -> Score;
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum GlobalConstraintType {
    TransportCost(TransportCostConstraint),
}

impl GlobalConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            Self::TransportCost(_) => "transport_cost",
        }
    }
}

impl GlobalConstraint for GlobalConstraintType {
    fn score_level(&self) -> ScoreLevel {
        match self {
            Self::TransportCost(constraint) => constraint.score_level(),
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            Self::TransportCost(constraint) => constraint.compute_insertion_score(context),
        }
    }

    fn compute_score(&self, context: &WorkingSolution) -> Score {
        match self {
            Self::TransportCost(constraint) => constraint.compute_score(context),
        }
    }
}
