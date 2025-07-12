use super::{recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams};

#[derive(Clone, Debug)]
pub struct SolverParams {
    pub max_iterations: usize,
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
}

#[derive(Clone, Debug)]
pub enum SolverSelectorStrategy {
    SelectBest,
    SelectRandom,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            max_solutions: 50,
            solver_acceptor: SolverAcceptorStrategy::Greedy,
            solver_selector: SolverSelectorStrategy::SelectBest,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
            threads: Threads::Multi(4),
        }
    }
}
