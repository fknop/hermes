use jiff::{SignedDuration, Timestamp};

use super::{score::Score, working_solution::WorkingSolution};

pub struct AcceptedSolution<'a> {
    pub solution: WorkingSolution<'a>,
    pub score: Score,
}
