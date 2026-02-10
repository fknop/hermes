use super::{
    score::{Score, ScoreAnalysis},
    solution::working_solution::WorkingSolution,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct AcceptedSolutionId(usize);

impl AcceptedSolutionId {
    pub fn new(id: usize) -> Self {
        AcceptedSolutionId(id)
    }
}

#[derive(Clone)]
pub struct AcceptedSolution {
    pub id: AcceptedSolutionId,
    pub solution: WorkingSolution,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}

impl AcceptedSolution {
    pub fn is_feasible(&self) -> bool {
        !self.score.is_infeasible()
    }
}
