use std::sync::Arc;

use jiff::Timestamp;
use parking_lot::{MappedRwLockReadGuard, RwLock};
use schemars::JsonSchema;
use serde::Serialize;

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

#[derive(Copy, Clone, Debug, Serialize, JsonSchema)]
pub enum SolverStatus {
    Pending,
    Running,
    Completed,
}

pub struct Solver {
    search: AlnsSearch,
    status: RwLock<SolverStatus>,
    created_at: Timestamp,
}

impl Solver {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        let search = AlnsSearch::new(params, Arc::new(problem));

        Solver {
            status: RwLock::new(SolverStatus::Pending),
            search,
            created_at: Timestamp::now(),
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

    pub fn created_at(&self) -> Timestamp {
        self.created_at
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
