use std::{
    cmp::Ordering,
    iter,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use fxhash::FxHashMap;
use schemars::JsonSchema;
use serde::Serialize;

use super::score_level::ScoreLevel;

pub const RUN_SCORE_ASSERTIONS: bool = false;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Score {
    pub hard_score: f64,
    pub soft_score: f64,
}

impl Score {
    pub const MAX: Score = Score {
        hard_score: f64::MAX,
        soft_score: f64::MAX,
    };

    pub const MIN: Score = Score {
        hard_score: f64::MIN,
        soft_score: f64::MIN,
    };

    pub const ZERO: Score = Score {
        hard_score: 0.0,
        soft_score: 0.0,
    };

    pub fn new(hard_score: f64, soft_score: f64) -> Self {
        Score {
            hard_score,
            soft_score,
        }
    }

    pub fn of(level: ScoreLevel, score: f64) -> Self {
        match level {
            ScoreLevel::Hard => Score::hard(score),
            ScoreLevel::Soft => Score::soft(score),
        }
    }

    pub fn hard(hard_score: f64) -> Self {
        Score {
            hard_score,
            soft_score: 0.0,
        }
    }

    pub fn soft(soft_score: f64) -> Self {
        Score {
            hard_score: 0.0,
            soft_score,
        }
    }

    pub fn zero() -> Self {
        Score {
            hard_score: 0.0,
            soft_score: 0.0,
        }
    }

    pub fn round(&self) -> Self {
        Score {
            hard_score: self.hard_score.round(),
            soft_score: self.soft_score.round(),
        }
    }

    pub fn is_failure(&self) -> bool {
        self.hard_score > 0.0
    }
}

impl Eq for Score {}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hard_score
            .total_cmp(&other.hard_score)
            .then_with(|| self.soft_score.total_cmp(&other.soft_score))
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl iter::Sum for Score {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, score| Score {
            hard_score: acc.hard_score + score.hard_score,
            soft_score: acc.soft_score + score.soft_score,
        })
    }
}

impl Add<Score> for Score {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Score {
            hard_score: self.hard_score + other.hard_score,
            soft_score: self.soft_score + other.soft_score,
        }
    }
}

impl AddAssign<Score> for Score {
    fn add_assign(&mut self, other: Score) {
        self.hard_score += other.hard_score;
        self.soft_score += other.soft_score;
    }
}

impl Sub<Score> for Score {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Score {
            hard_score: self.hard_score - other.hard_score,
            soft_score: self.soft_score - other.soft_score,
        }
    }
}

impl Div<f64> for Score {
    type Output = Self;

    fn div(self, divisor: f64) -> Self::Output {
        Score {
            hard_score: self.hard_score / divisor,
            soft_score: self.soft_score / divisor,
        }
    }
}

impl Mul<f64> for Score {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self::Output {
        Score {
            hard_score: self.hard_score * scalar,
            soft_score: self.soft_score * scalar,
        }
    }
}

impl SubAssign<Score> for Score {
    fn sub_assign(&mut self, other: Score) {
        self.hard_score -= other.hard_score;
        self.soft_score -= other.soft_score;
    }
}

#[derive(Default, Clone, Debug, Serialize, JsonSchema)]
pub struct ScoreAnalysis {
    pub scores: FxHashMap<&'static str, Score>,
}

impl ScoreAnalysis {
    pub fn total_score(&self) -> Score {
        self.scores.values().copied().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_addition() {
        let score1 = Score::hard(10.0);
        let score2 = Score::soft(5.0);
        let result = score1 + score2;
        assert_eq!(result.hard_score, 10.0);
        assert_eq!(result.soft_score, 5.0);
    }

    #[test]
    fn test_score_subtraction() {
        let score1 = Score::hard(10.0);
        let score2 = Score::soft(5.0);
        let result = score1 - score2;
        assert_eq!(result.hard_score, 10.0);
        assert_eq!(result.soft_score, -5.0);
    }

    #[test]
    fn test_score_sum() {
        let scores = vec![Score::hard(10.0), Score::soft(5.0), Score::hard(-3.0)];
        let total: Score = scores.into_iter().sum();
        assert_eq!(total.hard_score, 7.0);
        assert_eq!(total.soft_score, 5.0);
    }

    #[test]
    fn test_score_cmp() {
        let score1 = Score::hard(10.0);
        let score2 = Score::soft(5.0);
        let score3 = Score::hard(10.0);
        let score4 = Score::soft(15.0);
        let score5 = Score::new(2.0, 2.0);

        assert!(score1 > score2);
        assert!(score1 == score3);
        assert!(score2 < score3);

        assert!(score2 < score4);
        assert!(score1 > score4);

        assert!(score5 < score1);
        assert!(score5 < score3);
        assert!(score5 > score2);

        let vector = [score1, score2, score3, score4, score5];
        let max = vector.iter().max_by_key(|&score| score);
        assert_eq!(max, Some(&score1));

        assert_eq!(
            Score::new(20.0, 10.0).cmp(&Score::new(25.0, 100.0)),
            Ordering::Less
        );

        assert!(Score::soft(828.9368669428342) <= Score::soft(828.94));

        assert!((Score::soft(1788382.5109717606) >= Score::soft(1788382.5109717606)));
    }
}
