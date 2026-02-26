use std::sync::Arc;

use jiff::Timestamp;
use parking_lot::RwLock;
use schemars::JsonSchema;
use serde::Serialize;

#[cfg(feature = "statistics")]
use crate::solver::statistics::SearchStatistics;
use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        alns::AlnsRunResult, alns_weights::AlnsWeights,
        recreate::recreate_strategy::RecreateStrategy, ruin::ruin_strategy::RuinStrategy,
    },
};

use super::{accepted_solution::AcceptedSolution, alns::Alns, solver_params::SolverParams};

#[derive(Copy, Clone, Debug, Serialize, JsonSchema)]
pub enum SolverStatus {
    Pending,
    Running,
    Completed,
    Error,
}

pub struct Solver {
    search: Alns,
    status: RwLock<SolverStatus>,
    created_at: Timestamp,
}

impl Solver {
    pub fn new(problem: VehicleRoutingProblem, params: SolverParams) -> Self {
        let search = Alns::new(params, Arc::new(problem));

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

    pub fn solve(&self) -> anyhow::Result<AlnsRunResult> {
        *self.status.write() = SolverStatus::Running;
        match self.search.run() {
            Ok(result) => {
                *self.status.write() = SolverStatus::Completed;
                Ok(result)
            }
            Err(err) => {
                *self.status.write() = SolverStatus::Error;
                Err(err)
            }
        }
    }

    pub fn stop(&self) {
        self.search.stop();
        *self.status.write() = SolverStatus::Completed;
    }

    pub fn problem(&self) -> &Arc<VehicleRoutingProblem> {
        self.search.problem()
    }

    pub fn status(&self) -> SolverStatus {
        *self.status.read()
    }

    pub fn created_at(&self) -> Timestamp {
        self.created_at
    }

    pub fn current_best_solution(&self) -> Option<AcceptedSolution> {
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
