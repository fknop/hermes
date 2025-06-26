use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_random::RuinRandom, ruin_solution::RuinSolution};

#[derive(Clone, Copy, Debug)]
pub enum RuinStrategy {
    Random,
}

impl RuinSolution for RuinStrategy {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        match self {
            RuinStrategy::Random => {
                let strategy = RuinRandom;
                strategy.ruin_solution(solution, context);
            }
        }
    }
}
