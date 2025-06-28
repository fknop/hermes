use std::{
    cmp::Ordering,
    iter,
    ops::{Add, AddAssign},
};

use fxhash::FxHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Default, Debug)]
pub struct ScoreAnalysis {
    pub scores: FxHashMap<&'static str, Score>,
}

impl ScoreAnalysis {
    pub fn total_score(&self) -> Score {
        self.scores.values().copied().sum()
    }
}
