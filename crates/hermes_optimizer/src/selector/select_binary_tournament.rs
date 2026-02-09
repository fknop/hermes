use rand::{
    rngs::SmallRng,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::solver::accepted_solution::AcceptedSolution;

use super::select_solution::SelectSolution;

pub struct BinaryTournamentSelector;

impl SelectSolution for BinaryTournamentSelector {
    fn select_solution<'a>(
        &self,
        solutions: &'a [AcceptedSolution],
        rng: &mut impl rand::Rng,
    ) -> Option<&'a AcceptedSolution> {
        if solutions.is_empty() {
            return None; // No solutions to select from
        }

        if solutions.len() == 1 {
            return Some(&solutions[0]);
        }

        let solutions = solutions.iter().choose_multiple(rng, 2);
        let first = solutions[0];
        let second = solutions[1];

        if first.score < second.score {
            Some(first)
        } else {
            Some(second)
        }
    }
}
