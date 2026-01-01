use crate::solver::{
    insertion_context::InsertionContext, score::Score, score_level::ScoreLevel,
    solution::working_solution::WorkingSolution,
};

use super::global_constraint::GlobalConstraint;

pub struct UnassignedJobConstraint;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Hard;

impl GlobalConstraint for UnassignedJobConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        Score::of(
            self.score_level(),
            solution.unassigned_jobs().len() as f64 * solution.problem().unassigned_job_cost(),
        )
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        Score::zero()
    }
}
