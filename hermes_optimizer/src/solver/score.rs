use std::cmp::Ordering;

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
