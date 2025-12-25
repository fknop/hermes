use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    accepted_solution::AcceptedSolution,
    solver::{Solver, SolverStatus},
    solver_params::SolverParams,
    statistics::SearchStatistics,
};

#[derive(Default)]
pub struct SolverManager {
    solvers: RwLock<HashMap<String, Arc<Solver>>>, // This struct will manage the solver instances and their configurations
}

impl SolverManager {
    pub async fn solve(&self, job_id: String, problem: VehicleRoutingProblem) {
        let solver = Arc::new(Solver::new(problem, SolverParams::default()));
        self.solvers
            .write()
            .await
            .insert(job_id, Arc::clone(&solver));

        tokio::spawn(async move {
            solver.solve();
        });
    }

    pub async fn get_status(&self, job_id: &str) -> Option<SolverStatus> {
        self.solvers
            .read()
            .await
            .get(job_id)
            .map(|solver| solver.status())
    }

    pub async fn stop(&self, job_id: &str) {
        if let Some(solver) = self.solvers.write().await.remove(job_id) {
            solver.stop();
        }
    }

    pub async fn get_solution(&self, job_id: &str) -> Option<AcceptedSolution> {
        self.solvers
            .read()
            .await
            .get(job_id)
            .and_then(|solver| solver.current_best_solution())
            .map(|solution| solution.clone())
    }

    pub async fn get_statistics(&self, job_id: &str) -> Option<Arc<SearchStatistics>> {
        self.solvers
            .read()
            .await
            .get(job_id)
            .map(|solver| solver.statistics())
    }
}
