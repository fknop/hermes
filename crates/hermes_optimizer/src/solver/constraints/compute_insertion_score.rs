use crate::solver::{insertion_context::InsertionContext, score::Score, score_level::ScoreLevel};

use super::constraint::Constraint;

pub fn compute_insertion_score(
    constraints: &[Constraint],
    context: &InsertionContext,
    best_score: Option<&Score>,
    insert_on_failure: bool,
) -> Score {
    let mut score = Score::zero();

    let skip_on_failure = !insert_on_failure
        || best_score
            .map(|best_score| !best_score.is_failure())
            .unwrap_or(false);

    for constraint in constraints
        .iter()
        .filter(|c| c.score_level() == ScoreLevel::Hard)
    {
        score += constraint.compute_insertion_score(context);

        if score.is_failure() && skip_on_failure {
            return score;
        }
    }

    for constraint in constraints
        .iter()
        .filter(|c| c.score_level() == ScoreLevel::Soft)
    {
        let c_score = constraint.compute_insertion_score(context);
        score += c_score;
    }

    score
}
