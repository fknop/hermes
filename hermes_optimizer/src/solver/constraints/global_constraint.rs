use crate::solver::{
    insertion_context::InsertionContext, score::Score, working_solution::WorkingSolution,
};

pub trait GlobalConstraint {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score;
}

pub enum GlobalConstraintType {}

impl GlobalConstraintType {
    pub fn constraint_name(&self) -> &'static str {
        match self {
            _ => panic!(),
        }
    }
}

impl GlobalConstraint for GlobalConstraintType {
    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        match self {
            _ => panic!(),
        }
    }
}
