use super::{constraint::GlobalConstraint, score::Score, working_solution::WorkingSolution};

pub struct MinimizeCostConstraint;

impl GlobalConstraint for MinimizeCostConstraint {
    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let mut cost = 0;
        for route in solution.routes() {
            cost += route.total_cost();
        }

        Score::soft(cost)
    }
}
