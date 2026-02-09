use crate::{
    selector::select_solution::SelectSolution, solver::accepted_solution::AcceptedSolution,
};

// TODO: experiment with principles from HGS, such as elitism and diversity preservation
pub struct Population {
    population_size: usize,
    feasible_solutions: Vec<AcceptedSolution>,
    infeasible_solutions: Vec<AcceptedSolution>,
}

impl Population {
    pub fn new(population_size: usize) -> Self {
        Population {
            population_size,
            feasible_solutions: Vec::with_capacity(population_size),
            infeasible_solutions: Vec::with_capacity(population_size),
        }
    }
}

impl Population {
    pub fn select_solution(
        &self,
        selector: &impl SelectSolution,
        rng: &mut impl rand::Rng,
    ) -> Option<&AcceptedSolution> {
        if !self.feasible_solutions.is_empty() {
            selector.select_solution(&self.feasible_solutions, rng)
        } else if !self.infeasible_solutions.is_empty() {
            selector.select_solution(&self.infeasible_solutions, rng)
        } else {
            None
        }
    }

    pub fn solutions(&self) -> &[AcceptedSolution] {
        if !self.feasible_solutions.is_empty() {
            &self.feasible_solutions
        } else if !self.infeasible_solutions.is_empty() {
            &self.infeasible_solutions
        } else {
            &[]
        }
    }

    pub fn infeasible_solutions(&self) -> &[AcceptedSolution] {
        &self.infeasible_solutions
    }

    pub fn feasible_solutions(&self) -> &[AcceptedSolution] {
        &self.feasible_solutions
    }

    pub fn all_solutions(&self) -> impl Iterator<Item = &AcceptedSolution> {
        self.feasible_solutions
            .iter()
            .chain(self.infeasible_solutions.iter())
    }

    pub fn add_solution(&mut self, accepted_solution: AcceptedSolution) {
        if accepted_solution.is_feasible() {
            Self::insert_solution(
                &mut self.feasible_solutions,
                accepted_solution,
                self.population_size,
            );
        } else {
            Self::insert_solution(
                &mut self.infeasible_solutions,
                accepted_solution,
                self.population_size,
            );
        }
    }

    pub fn is_empty(&self) -> bool {
        self.feasible_solutions.is_empty() && self.infeasible_solutions.is_empty()
    }

    pub fn best(&self) -> Option<&AcceptedSolution> {
        self.feasible_solutions
            .first()
            .or_else(|| self.infeasible_solutions.first())
    }

    // TODO: handle duplicates
    fn insert_solution(
        accepted_solutions: &mut Vec<AcceptedSolution>,
        new_accepted_solution: AcceptedSolution,
        population_size: usize,
    ) {
        let is_duplicate = accepted_solutions.iter().any(|accepted_solution| {
            accepted_solution.score == new_accepted_solution.score
                && accepted_solution
                    .solution
                    .is_identical(&new_accepted_solution.solution)
        });

        // We don't add it if duplicate to keep the population varied enough
        if is_duplicate {
            return;
        }

        if accepted_solutions.len() == population_size {
            accepted_solutions.pop();
        }

        match accepted_solutions.binary_search_by(|accepted_solution| {
            accepted_solution
                .solution
                .unassigned_jobs()
                .len()
                .cmp(&new_accepted_solution.solution.unassigned_jobs().len())
                .then(accepted_solution.score.cmp(&new_accepted_solution.score))
        }) {
            Ok(pos) | Err(pos) => {
                accepted_solutions.insert(pos, new_accepted_solution);
            }
        }
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
        let mut population = Population::new(3);

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

        let solution1 = AcceptedSolution {
            solution: one_unassigned_solution.clone(),
            score: Score::soft(10.0),
            score_analysis: ScoreAnalysis::default(),
        };

        population.add_solution(solution1);

        assert_eq!(population.feasible_solutions.len(), 1);

        let solution2 = AcceptedSolution {
            solution: one_unassigned_solution.clone(),
            score: Score::soft(15.0),
            score_analysis: ScoreAnalysis::default(),
        };

        population.add_solution(solution2);

        assert_eq!(population.feasible_solutions.len(), 2);
        assert_eq!(population.feasible_solutions[0].score, Score::soft(10.0));
        assert_eq!(population.feasible_solutions[1].score, Score::soft(15.0));

        let no_unassigned_solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![test_utils::TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            }],
        );

        let solution3 = AcceptedSolution {
            solution: no_unassigned_solution.clone(),
            score: Score::soft(20.0),
            score_analysis: ScoreAnalysis::default(),
        };
        population.add_solution(solution3);

        assert_eq!(population.feasible_solutions.len(), 3);
        assert_eq!(population.feasible_solutions[0].score, Score::soft(20.0));
        assert_eq!(population.feasible_solutions[1].score, Score::soft(10.0));
        assert_eq!(population.feasible_solutions[2].score, Score::soft(15.0));
    }
}
