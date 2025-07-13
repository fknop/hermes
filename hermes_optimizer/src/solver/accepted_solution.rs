use jiff::{SignedDuration, Timestamp};

use super::{
    score::{Score, ScoreAnalysis},
    working_solution::WorkingSolution,
};

#[derive(Clone)]
pub struct AcceptedSolution {
    pub solution: WorkingSolution,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}
