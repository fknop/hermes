pub struct SolverParams {
    pub max_iterations: usize,
    pub solver_acceptor: SolverAcceptorType,
    pub solver_selector: SolverSelectorType,
}

pub enum SolverAcceptorType {
    Greedy,
}

pub enum SolverSelectorType {
    SelectBest,
    SelectRandom,
}
