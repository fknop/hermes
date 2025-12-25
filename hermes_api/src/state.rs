use hermes_optimizer::solver::solver_manager::SolverManager;
use hermes_routing::hermes::Hermes;

pub struct AppState {
    pub hermes: Hermes,
    pub solver_manager: SolverManager,
}
