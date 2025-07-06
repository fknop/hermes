use crate::solver::working_solution::WorkingSolution;

use super::{
    ruin_context::RuinContext, ruin_radial::RuinRadial, ruin_random::RuinRandom,
    ruin_solution::RuinSolution, ruin_worst::RuinWorst,
};

#[derive(Clone, Copy, Debug)]
pub enum RuinStrategy {
    Random,
    RuinRadial,
    RuinWorst,
}

impl RuinSolution for RuinStrategy {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        match self {
            RuinStrategy::Random => {
                let strategy = RuinRandom;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinRadial => {
                let strategy = RuinRadial;
                strategy.ruin_solution(solution, context);
            }
            RuinStrategy::RuinWorst => {
                let strategy = RuinWorst;
                strategy.ruin_solution(solution, context);
            }
        }
    }
}
