use super::ruin::ruin_params::RuinParams;

pub struct SolverParams {
    pub max_iterations: usize,
    pub solver_acceptor: SolverAcceptorType,
    pub solver_selector: SolverSelectorType,

    pub ruin: RuinParams,
}

pub enum SolverAcceptorType {
    Greedy,
}

pub enum SolverSelectorType {
    SelectBest,
    SelectRandom,
}
