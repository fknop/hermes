use std::collections::HashMap;

use super::solver::Solver;

pub struct SolverManager<'a> {
    solvers: HashMap<String, Solver<'a>>, // This struct will manage the solver instances and their configurations
}

impl SolverManager<'_> {
    pub fn new() -> Self {
        SolverManager {
            solvers: HashMap::new(),
        }
    }

    // pub fn add_solver(&mut self, name: String, solver: Solver) {
    // self.solvers.insert(name, solver);
    // }
}
