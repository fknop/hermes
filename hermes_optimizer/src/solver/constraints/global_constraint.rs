use crate::solver::{insertion_context::InsertionContext, score::Score};

use super::transport_cost_constraint::TransportCostConstraint;

pub trait GlobalConstraint {
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
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            Self::TransportCost(constraint) => constraint.compute_insertion_score(context),
        }
    }
}
