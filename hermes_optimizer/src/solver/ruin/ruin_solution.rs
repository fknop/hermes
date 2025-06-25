use crate::solver::working_solution::WorkingSolution;

pub trait RuinSolution {
    fn ruin_solution(&self, solution: &mut WorkingSolution, num_activities_to_remove: usize);
}
