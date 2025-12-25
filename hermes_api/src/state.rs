use hermes_optimizer_core::solver::solver_manager::SolverManager;
use hermes_routing_core::hermes::Hermes;

pub struct AppState {
    pub hermes: Hermes,
    pub solver_manager: SolverManager,
}
