use crate::solver::{score::Score, working_solution::WorkingSolution};

pub trait GlobalConstraint {
    fn compute_delta_score(&self, solution: &WorkingSolution) -> Score;
}

pub enum GlobalConstraintType {}

impl GlobalConstraint for GlobalConstraintType {
    fn compute_delta_score(&self, _solution: &WorkingSolution) -> Score {
        match self {
            _ => Score::zero(),
        }
    }
}
