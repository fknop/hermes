use super::{recreate::recreate_params::RecreateParams, ruin::ruin_params::RuinParams};

pub struct SolverParams {
    pub max_iterations: usize,
    pub solver_acceptor: SolverAcceptorStrategy,
    pub solver_selector: SolverSelectorStrategy,

    pub ruin: RuinParams,
    pub recreate: RecreateParams,
}

pub enum SolverAcceptorStrategy {
    Greedy,
}

pub enum SolverSelectorStrategy {
    SelectBest,
    SelectRandom,
}

impl Default for SolverParams {
    fn default() -> Self {
        Self {
            max_iterations: 1000,
            solver_acceptor: SolverAcceptorStrategy::Greedy,
            solver_selector: SolverSelectorStrategy::SelectBest,
            ruin: RuinParams::default(),
            recreate: RecreateParams::default(),
        }
    }
}
