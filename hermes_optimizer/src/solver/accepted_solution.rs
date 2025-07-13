use jiff::{SignedDuration, Timestamp};

use super::{
    score::{Score, ScoreAnalysis},
    working_solution::WorkingSolution,
};

#[derive(Clone)]
pub struct AcceptedSolution<'a> {
    pub solution: WorkingSolution<'a>,
    pub score: Score,
    pub score_analysis: ScoreAnalysis,
}
