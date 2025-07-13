use hermes_core::hermes::Hermes;
use hermes_optimizer::solver::solver_manager::SolverManager;

pub struct AppState {
    pub hermes: Hermes,
    pub solver_manager: SolverManager,
}
