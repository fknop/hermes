//! Slack Induction by String Removals for Vehicle Routing Problems
//! Jan Christiaens, Greet Vanden Berghe

use fxhash::FxHashSet;
use rand::{Rng, rngs::SmallRng, seq::IndexedRandom};

use crate::solver::working_solution::WorkingSolution;

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

pub struct RuinString {
    /// Min numbers of string to ruin
    k_min: usize,

    /// Max numbers of string to ruin
    k_max: usize,

    /// Minimum length of string to ruin
    l_min: usize,

    /// Maximum length of string to ruin
    l_max: usize,
}

impl Default for RuinString {
    fn default() -> Self {
        RuinString {
            k_min: 1,
            k_max: 6,
            l_min: 3,
            l_max: 20,
        }
    }
}

impl RuinString {
    fn compute_possible_string_start(
        string_length: usize,
        index: usize,
        route_length: usize,
    ) -> Vec<usize> {
        let mut starts = vec![];
        for i in 1..=string_length {
            let lower: i64 = index as i64 - (string_length as i64 - i as i64);
            let upper = index + (i - 1);
            if lower >= 0 && upper < route_length {
                starts.push(lower as usize);
            }
        }

        starts
    }

    fn compute_preserved_length(
        string_length: usize,
        route_length: usize,
        rng: &mut SmallRng,
    ) -> usize {
        // Cannot preserve anything in this case
        if string_length == route_length {
            return 0;
        }

        let mut preserved_length = 1;
        while string_length + preserved_length < route_length {
            if rng.random_bool(0.01) {
                return preserved_length;
            } else {
                preserved_length += 1;
            }
        }

        preserved_length
    }

    fn ruin_string(&self, solution: &mut WorkingSolution, rng: &mut SmallRng, route_id: usize) {
        let route = solution.route(route_id);
        let route_length = route.activities().len();
        let string_length = rng.random_range(self.l_min..=self.l_max).min(route_length);

        let random_activity = rng.random_range(0..route_length);
        let possible_starts =
            Self::compute_possible_string_start(string_length, random_activity, route_length);
        if possible_starts.is_empty() {
            return; // No valid starting point for the string
        }

        let start = possible_starts.choose(rng).cloned().unwrap();

        for i in start..(start + string_length) {
            solution.remove_activity(route_id, i);
        }
    }

    fn ruin_split_string(
        &self,
        solution: &mut WorkingSolution,
        rng: &mut SmallRng,
        route_id: usize,
    ) {
        let route = solution.route(route_id);
        let route_length = route.activities().len();
        let string_length = rng.random_range(self.l_min..=self.l_max).min(route_length);
        let preserved_string_length =
            Self::compute_preserved_length(string_length, route_length, rng);

        let total_string_length = string_length + preserved_string_length;

        let random_activity = rng.random_range(0..route_length);
        let possible_starts =
            Self::compute_possible_string_start(total_string_length, random_activity, route_length);
        if possible_starts.is_empty() {
            return; // No valid starting point for the string
        }

        let start = possible_starts.choose(rng).cloned().unwrap();
        let start_of_preserved_string = rng.random_range(0..string_length);

        let mut string_position = 0;

        for i in start..(start + total_string_length) {
            if string_position >= start_of_preserved_string
                && string_position < start_of_preserved_string + preserved_string_length
            {
                string_position += 1;
                continue;
            }

            solution.remove_activity(route_id, i);
            string_position += 1;
        }
    }
}

impl RuinSolution for RuinString {
    fn ruin_solution(&self, solution: &mut WorkingSolution, context: RuinContext) {
        let k = context
            .rng
            .random_range(self.k_min..=self.k_max)
            .min(solution.routes().len());

        let mut ruined_routes = FxHashSet::<usize>::default();
        let mut seed_service = context
            .rng
            .random_range(0..context.problem.services().len());

        while ruined_routes.len() < k {
            let route_id = solution.route_of_service(seed_service);

            if let Some(route_id) = route_id {
                if context.rng.random_bool(0.5) {
                    self.ruin_string(solution, context.rng, route_id);
                } else {
                    self.ruin_split_string(solution, context.rng, route_id);
                }

                ruined_routes.insert(route_id);
            }

            let nearest_service_of_different_route = context
                .problem
                .nearest_services(seed_service)
                .find(|&service_id| {
                    if let Some(route_id) = solution.route_of_service(service_id) {
                        !ruined_routes.contains(&route_id)
                    } else {
                        false
                    }
                });

            if let Some(service_id) = nearest_service_of_different_route {
                seed_service = service_id;
            } else {
                // No more services to ruin, break the loop
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_possible_string_start() {
        assert_eq!(
            RuinString::compute_possible_string_start(3, 1, 5),
            vec![0, 1]
        );

        assert_eq!(RuinString::compute_possible_string_start(3, 4, 5), vec![2]);
    }
}
