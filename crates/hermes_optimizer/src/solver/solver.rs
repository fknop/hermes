use std::sync::Arc;

use parking_lot::{MappedRwLockReadGuard, RwLock};

use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        alns_weights::AlnsWeights, recreate::recreate_strategy::RecreateStrategy,
        ruin::ruin_strategy::RuinStrategy,
    },
};

use super::{
    accepted_solution::AcceptedSolution, alns_search::AlnsSearch, solver_params::SolverParams,
    statistics::SearchStatistics,
};

#[derive(Copy, Clone, Debug)]
pub enum SolverStatus {
    Pending,
    Running,
    Completed,
}

pub struct Solver {
    search: AlnsSearch,
    status: RwLock<SolverStatus>,
}

impl Solver {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        let search = AlnsSearch::new(params, Arc::new(problem));

        Solver {
            status: RwLock::new(SolverStatus::Pending),
            search,
        }
    }

    pub fn on_best_solution<F>(&mut self, callback: F)
    where
        F: FnMut(&AcceptedSolution) + Send + Sync + 'static,
    {
        self.search.on_best_solution(callback);
    }

    pub fn solve(&self) {
        *self.status.write() = SolverStatus::Running;
        self.search.run();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn stop(&self) {
        self.search.stop();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn status(&self) -> SolverStatus {
        *self.status.read()
    }

    pub fn current_best_solution(&self) -> Option<MappedRwLockReadGuard<'_, AcceptedSolution>> {
        self.search.best_solution()
    }

    #[cfg(feature = "statistics")]
    pub fn statistics(&self) -> Arc<SearchStatistics> {
        self.search.statistics()
    }

    pub fn weights(&self) -> (AlnsWeights<RuinStrategy>, AlnsWeights<RecreateStrategy>) {
        self.search.weights_cloned()
    }
}

#[cfg(test)]
mod tests {}
