use crate::solver::working_solution::WorkingSolution;

pub trait RecreateSolution {
    fn recreate_solution(&self, solution: &mut WorkingSolution);
}
