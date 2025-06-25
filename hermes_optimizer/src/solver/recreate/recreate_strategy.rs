use crate::solver::working_solution::WorkingSolution;

use super::{random_insertion::RandomInsertion, recreate_solution::RecreateSolution};

#[derive(Clone, Copy, Debug)]
pub enum RecreateStrategy {
    RandomInsertion,
    BestInsertion,
}

impl RecreateSolution for RecreateStrategy {
    fn recreate_solution(&self, solution: &mut WorkingSolution) {
        match self {
            RecreateStrategy::RandomInsertion => {
                let strategy = RandomInsertion;
                strategy.recreate_solution(solution);
            }
            RecreateStrategy::BestInsertion => {
                todo!()
            }
        }
    }
}
