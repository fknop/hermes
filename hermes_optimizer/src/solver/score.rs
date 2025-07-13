use std::{
    cmp::Ordering,
    iter,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use fxhash::FxHashMap;
use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct Score {
    pub hard_score: i64,
    pub soft_score: i64,
}

impl Score {
    pub const MAX: Score = Score {
        hard_score: i64::MAX,
        soft_score: i64::MAX,
    };

    pub const MIN: Score = Score {
        hard_score: i64::MIN,
        soft_score: i64::MIN,
    };

    pub fn new(hard_score: i64, soft_score: i64) -> Self {
        Score {
            hard_score,
            soft_score,
        }
    }

    pub fn hard(hard_score: i64) -> Self {
        Score {
            hard_score,
            soft_score: 0,
        }
    }

    pub fn soft(soft_score: i64) -> Self {
        Score {
            hard_score: 0,
            soft_score,
        }
    }

    pub fn zero() -> Self {
        Score {
            hard_score: 0,
            soft_score: 0,
        }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hard_score
            .cmp(&other.hard_score)
            .then_with(|| self.soft_score.cmp(&other.soft_score))
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

impl SubAssign<Score> for Score {
    fn sub_assign(&mut self, other: Score) {
        self.hard_score -= other.hard_score;
        self.soft_score -= other.soft_score;
    }
}

#[derive(Default, Clone, Debug, Serialize)]
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
        let score1 = Score::hard(10);
        let score2 = Score::soft(5);
        let result = score1 + score2;
        assert_eq!(result.hard_score, 10);
        assert_eq!(result.soft_score, 5);
    }

    #[test]
    fn test_score_subtraction() {
        let score1 = Score::hard(10);
        let score2 = Score::soft(5);
        let result = score1 - score2;
        assert_eq!(result.hard_score, 10);
        assert_eq!(result.soft_score, -5);
    }

    #[test]
    fn test_score_sum() {
        let scores = vec![Score::hard(10), Score::soft(5), Score::hard(-3)];
        let total: Score = scores.into_iter().sum();
        assert_eq!(total.hard_score, 7);
        assert_eq!(total.soft_score, 5);
    }

    #[test]
    fn test_score_cmp() {
        let score1 = Score::hard(10);
        let score2 = Score::soft(5);
        let score3 = Score::hard(10);
        let score4 = Score::soft(15);

        let score5 = Score::new(2, 2);

        assert!(score1 > score2);
        assert!(score1 == score3);
        assert!(score2 < score3);

        assert!(score2 < score4);
        assert!(score1 > score4);

        assert!(score5 < score1);
        assert!(score5 < score3);
        assert!(score5 > score2);
    }
}
