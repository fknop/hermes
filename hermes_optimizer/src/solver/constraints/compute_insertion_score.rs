use crate::solver::{insertion_context::InsertionContext, score::Score, score_level::ScoreLevel};

use super::constraint::Constraint;

pub fn compute_insertion_score(
    constraints: &[Constraint],
    context: &InsertionContext,
    best_score: Option<&Score>,
) -> Score {
    let mut score = Score::zero();

    let skip_hard_failure = best_score
        .map(|best_score| best_score.hard_score == 0.0)
        .unwrap_or(false);

    for constraint in constraints
        .iter()
        .filter(|c| c.score_level() == ScoreLevel::Hard)
    {
        score += constraint.compute_insertion_score(context);

        // if score.hard_score > 0.0 && skip_hard_failure {
        //     return score;
        // }
    }

    for constraint in constraints
        .iter()
        .filter(|c| c.score_level() == ScoreLevel::Soft)
    {
        score += constraint.compute_insertion_score(context);
    }

    return score;
}
