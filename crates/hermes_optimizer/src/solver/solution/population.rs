use std::{collections::BTreeMap, sync::atomic::AtomicUsize};

use fxhash::FxHashMap;
use tracing::info;

use crate::{
    selector::select_solution::SelectSolution,
    solver::{
        accepted_solution::{AcceptedSolution, AcceptedSolutionId},
        score::{Score, ScoreAnalysis},
        solution::working_solution::WorkingSolution,
        solver_params::PopulationParams,
    },
};

// TODO: experiment with principles from HGS, such as elitism and diversity preservation
pub struct Population {
    id_counter: AtomicUsize,
    params: PopulationParams,
    solutions: Vec<AcceptedSolution>,
    broken_pair_distances: FxHashMap<AcceptedSolutionId, BTreeMap<usize, AcceptedSolutionId>>,
    biased_fitnesses: Vec<f64>,
}

impl Population {
    pub fn new(params: PopulationParams) -> Self {
        Population {
            id_counter: AtomicUsize::new(0),
            broken_pair_distances: FxHashMap::default(),
            solutions: Vec::with_capacity(params.size),
            biased_fitnesses: Vec::with_capacity(params.size),
            params,
        }
    }
}

impl Population {
    fn next_id(&self) -> usize {
        self.id_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn select_solution(
        &self,
        selector: &impl SelectSolution,
        rng: &mut impl rand::Rng,
    ) -> Option<&AcceptedSolution> {
        if !self.solutions.is_empty() {
            selector.select_solution(self, rng)
        } else {
            None
        }
    }

    pub fn solutions(&self) -> &[AcceptedSolution] {
        &self.solutions
    }

    pub fn feasible_solutions(&self) -> &[AcceptedSolution] {
        &self.solutions
    }

    fn update_fitnesses(&mut self) {
        self.biased_fitnesses.resize(self.solutions.len(), 0.0);
        self.biased_fitnesses.fill(0.0);

        let mut rankings = self
            .solutions
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s, self.average_broken_pairs_distance(s)))
            .collect::<Vec<_>>();

        // Sort based on diversity contribution (decreasing distance)
        rankings.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        if self.solutions.len() == 1 {
            self.biased_fitnesses[0] = 0.0;
        } else {
            for (i, (rank, _, _)) in rankings.iter().enumerate() {
                let fit_rank = (*rank as f64) / self.solutions.len() as f64;
                let diversity_rank = (i as f64) / self.solutions.len() as f64;

                if self.solutions.len() < self.params.elite_size {
                    self.biased_fitnesses[*rank] = fit_rank;
                } else {
                    self.biased_fitnesses[*rank] = fit_rank
                        + (1.0 - (self.params.elite_size as f64 / self.solutions.len() as f64))
                            * diversity_rank;
                }
            }
        }
    }

    fn remove_worst_fitness(&mut self) -> Option<AcceptedSolution> {
        let worst_fitness = self
            .biased_fitnesses
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())?;

        let worst_index = self
            .biased_fitnesses
            .iter()
            .position(|&x| x == *worst_fitness)?;

        Some(self.solutions.remove(worst_index))
    }

    pub fn add_solution(
        &mut self,
        solution: WorkingSolution,
        score: Score,
        score_analysis: ScoreAnalysis,
    ) {
        let is_duplicate = self.solutions.iter().any(|accepted_solution| {
            accepted_solution.score == score && accepted_solution.solution.is_identical(&solution)
        });

        // We don't add it if duplicate to keep the population varied enough
        if is_duplicate {
            return;
        }

        #[allow(clippy::collapsible_if)] // I think it's clearer this way
        if self.solutions.len() == self.params.size {
            if let Some(removed_solution) = self.remove_worst_fitness() {
                // TODO: remove based on fitness value instead of worst
                // Cleanup data for removed solution
                self.broken_pair_distances.remove(&removed_solution.id);
                self.broken_pair_distances
                    .iter_mut()
                    .for_each(|(_, distances)| {
                        distances.retain(|_, v| *v != removed_solution.id);
                    });
            }
        }

        let id = AcceptedSolutionId::new(self.next_id());

        let new_accepted = AcceptedSolution {
            id,
            solution,
            score,
            score_analysis,
        };

        // Compute broken pair distance for new solution
        for accepted_solution in &self.solutions {
            let distance = accepted_solution
                .solution
                .broken_pairs_distance(&new_accepted.solution);

            self.broken_pair_distances
                .entry(id)
                .or_default()
                .insert(distance, accepted_solution.id);
            self.broken_pair_distances
                .entry(accepted_solution.id)
                .or_default()
                .insert(distance, id);
        }

        match self.solutions.binary_search_by(|accepted_solution| {
            accepted_solution
                .solution
                .unassigned_jobs()
                .len()
                .cmp(&new_accepted.solution.unassigned_jobs().len())
                .then(accepted_solution.score.cmp(&score))
        }) {
            Ok(pos) | Err(pos) => {
                self.solutions.insert(pos, new_accepted);
            }
        }

        self.update_fitnesses();
    }

    pub fn biased_fitness(&self, solution: &AcceptedSolution) -> f64 {
        if let Some(pos) = self.solutions.iter().position(|s| s.id == solution.id) {
            self.biased_fitnesses[pos]
        } else {
            0.0
        }
    }

    pub fn average_broken_pairs_distance(&self, accepted_solution: &AcceptedSolution) -> f64 {
        if let Some(distances) = self.broken_pair_distances.get(&accepted_solution.id) {
            let n_closest = self.params.n_closest();
            let sum = distances
                .iter()
                .take(n_closest)
                .map(|(k, _)| *k)
                .sum::<usize>();
            sum as f64 / n_closest as f64
        } else {
            0.0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.solutions.is_empty()
    }

    pub fn best(&self) -> Option<&AcceptedSolution> {
        self.solutions.first()
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use crate::{
        solver::score::{Score, ScoreAnalysis},
        test_utils,
    };

    use super::*;

    #[test]
    fn test_population() {
        let mut population = Population::new(PopulationParams {
            size: 3,
            ..PopulationParams::default()
        });

        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        // The actual solution don't really matter for the tests
        let one_unassigned_solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![test_utils::TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            }],
        );

        population.add_solution(
            one_unassigned_solution.clone(),
            Score::soft(10.0),
            ScoreAnalysis::default(),
        );

        assert_eq!(population.solutions.len(), 1);

        population.add_solution(
            one_unassigned_solution.clone(),
            Score::soft(15.0),
            ScoreAnalysis::default(),
        );

        assert_eq!(population.solutions.len(), 2);
        assert_eq!(population.solutions[0].score, Score::soft(10.0));
        assert_eq!(population.solutions[1].score, Score::soft(15.0));

        let no_unassigned_solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![test_utils::TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            }],
        );

        population.add_solution(
            no_unassigned_solution.clone(),
            Score::soft(20.0),
            ScoreAnalysis::default(),
        );

        assert_eq!(population.solutions.len(), 3);
        assert_eq!(population.solutions[0].score, Score::soft(20.0));
        assert_eq!(population.solutions[1].score, Score::soft(10.0));
        assert_eq!(population.solutions[2].score, Score::soft(15.0));
    }
}
