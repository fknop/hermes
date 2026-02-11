use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{
    solver::Solver,
    solver_params::SolverParams,
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

    pub async fn list_solvers(&self) -> Vec<(String, Arc<Solver>)> {
        let solvers = self.solvers.read().await;
        solvers
            .iter()
            .map(|(job_id, solver)| (job_id.clone(), Arc::clone(solver)))
            .collect()
    }

    pub async fn create_job(&self, problem: VehicleRoutingProblem) -> String {
        let job_id = problem.id().to_owned();
        let solver = Arc::new(Solver::new(problem, SolverParams::default()));
        self.solvers.write().await.insert(job_id.clone(), solver);
        job_id
    }

    pub async fn start(&self, job_id: &str) -> bool {
        if let Some(solver) = self.solvers.read().await.get(job_id).cloned() {
            std::thread::spawn(move || {
                solver.solve();
            });
            true
        } else {
            false
        }
    }

    pub async fn stop(&self, job_id: &str) -> bool {
        if let Some(solver) = self.solvers.read().await.get(job_id).cloned() {
            solver.stop();
            true
        } else {
            false
        }
    }

    pub async fn solver(&self, job_id: &str) -> Option<Arc<Solver>> {
        self.solvers.read().await.get(job_id).cloned()
    }
}
