use jiff::SignedDuration;

use super::{recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams};

#[derive(Clone, Debug)]
pub struct SolverParams {
    pub max_iterations: usize,
    pub max_duration: SignedDuration,
    pub solver_acceptor: SolverAcceptorStrategy,
    pub solver_selector: SolverSelectorStrategy,

    pub max_solutions: usize,

    pub ruin: RuinParams,
    pub recreate: RecreateParams,
    pub threads: Threads,
}

impl SolverParams {
    pub fn prepare(&mut self) {
        self.ruin
            .ruin_strategies
            .sort_by(|(_, w1), (_, w2)| w1.cmp(w2));

        self.recreate
            .recreate_strategies
            .sort_by(|(_, w1), (_, w2)| w1.cmp(w2));
    }
}

#[derive(Clone, Debug)]
pub enum Threads {
    Single,
    Auto,
    Multi(usize),
}

#[derive(Clone, Debug)]
pub enum SolverAcceptorStrategy {
    Greedy,
    Schrimpf,
}

#[derive(Clone, Debug)]
pub enum SolverSelectorStrategy {
    SelectBest,
    SelectRandom,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            max_iterations: 100000,
            max_duration: SignedDuration::from_mins(1),
            max_solutions: 20,
            solver_acceptor: SolverAcceptorStrategy::Schrimpf,
            solver_selector: SolverSelectorStrategy::SelectRandom,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            threads: Threads::Auto,
        }
    }
}
