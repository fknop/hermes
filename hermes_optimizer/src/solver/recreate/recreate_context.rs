use rand::rngs::SmallRng;

use crate::solver::{
    constraints::{compute_constraints_score::compute_insertion_score, constraint::Constraint},
    insertion::Insertion,
    score::Score,
    working_solution::{WorkingSolution, compute_insertion_context},
};

pub struct RecreateContext<'a> {
    pub rng: &'a mut SmallRng,
    pub constraints: &'a Vec<Constraint>,
}

impl<'a> RecreateContext<'a> {
    pub fn compute_insertion_score(
        &self,
        solution: &WorkingSolution,
        insertion: &Insertion,
    ) -> Score {
        let context = compute_insertion_context(solution.problem(), solution, insertion);
        compute_insertion_score(self.constraints, &context)
    }
}
