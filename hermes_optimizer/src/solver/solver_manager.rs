use std::collections::HashMap;

use uuid::Uuid;

use crate::problem::vehicle_routing_problem::VehicleRoutingProblem;

use super::{accepted_solution::AcceptedSolution, solver::Solver, solver_params::SolverParams};

#[derive(Default)]
pub struct SolverManager<'a> {
    solvers: HashMap<String, Solver<'a>>, // This struct will manage the solver instances and their configurations
}

impl SolverManager<'_> {
    pub fn solve(&mut self, problem: VehicleRoutingProblem) -> String {
        let job_id = Uuid::new_v4().to_string();

        let solver = Solver::new(problem, SolverParams::default());
        self.solvers.insert(job_id.clone(), solver);

        job_id
    }

    // TODO: avoid cloning the solution
    pub fn get_best_solution(&self, job_id: String) -> Option<AcceptedSolution<'_>> {
        if let Some(solver) = self.solvers.get(&job_id) {
            let guard = solver.best_solutions();
            guard.first().cloned()
        } else {
            None
        }
    }
}
