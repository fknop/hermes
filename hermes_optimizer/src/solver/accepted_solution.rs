use serde::Serialize;

use super::{
    score::{Score, ScoreAnalysis},
    working_solution::WorkingSolution,
};

#[derive(Clone, Serialize)]
pub struct AcceptedSolution {
    #[serde(flatten)]
    pub solution: WorkingSolution,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}
