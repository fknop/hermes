use super::{
    score::{Score, ScoreAnalysis},
    solution::working_solution::WorkingSolution,
};

#[derive(Clone)]
pub struct AcceptedSolution {
    pub solution: WorkingSolution,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}

impl AcceptedSolution {
    pub fn is_feasible(&self) -> bool {
        !self.score.is_infeasible()
    }
}
