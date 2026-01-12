//! Slack Induction by String Removals for Vehicle Routing Problems
//! Jan Christiaens, Greet Vanden Berghe

use fxhash::FxHashSet;
use rand::seq::IndexedRandom;

use crate::{
    problem::job::ActivityId,
    solver::solution::{route_id::RouteIdx, working_solution::WorkingSolution},
};

use super::{ruin_context::RuinContext, ruin_solution::RuinSolution};

// TODO: support shipments
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
            k_max: 3,
            l_min: 3,
            l_max: 10,
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

    fn compute_preserved_length<R>(string_length: usize, route_length: usize, rng: &mut R) -> usize
    where
        R: rand::Rng,
    {
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

    fn ruin_string<R>(&self, solution: &mut WorkingSolution, rng: &mut R, route_id: RouteIdx)
    where
        R: rand::Rng,
    {
        let route = solution.route(route_id);
        let route_length = route.activity_ids().len();
        let string_length = rng.random_range(self.l_min..=self.l_max).min(route_length);

        let random_activity = rng.random_range(0..route_length);
        let possible_starts =
            Self::compute_possible_string_start(string_length, random_activity, route_length);
        if possible_starts.is_empty() {
            return; // No valid starting point for the string
        }

        let start = possible_starts.choose(rng).cloned().unwrap();

        for _ in start..(start + string_length) {
            // Always remove the start, as the start+1 becomes the start once start is removed
            solution.remove_activity(route_id, start);
        }
    }

    fn ruin_split_string<R>(&self, solution: &mut WorkingSolution, rng: &mut R, route_id: RouteIdx)
    where
        R: rand::Rng,
    {
        let route = solution.route(route_id);
        let route_length = route.activity_ids().len();
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

        self.remove_split_string(
            solution,
            RuinSplitStringParams {
                route_id,
                start,
                start_of_preserved_string,
                string_length,
                preserved_string_length,
            },
        );
    }

    fn remove_split_string(
        &self,
        solution: &mut WorkingSolution,
        RuinSplitStringParams {
            route_id,
            start,
            start_of_preserved_string,
            string_length,
            preserved_string_length,
        }: RuinSplitStringParams,
    ) {
        let route_len = solution.route(route_id).len();
        assert!(start_of_preserved_string < string_length);
        assert!(start < route_len);

        let total_string_length = string_length + preserved_string_length;

        assert!(total_string_length <= route_len);

        let mut preserved = 0;

        for string_position in 0..total_string_length {
            if string_position >= start_of_preserved_string
                && string_position < start_of_preserved_string + preserved_string_length
            {
                preserved += 1;
                continue;
            }

            // s, s+1, p, p+1, s+4

            solution.remove_activity(route_id, start + preserved);
        }
    }
}

struct RuinSplitStringParams {
    route_id: RouteIdx,
    start: usize,
    start_of_preserved_string: usize,
    string_length: usize,
    preserved_string_length: usize,
}

impl RuinSolution for RuinString {
    fn ruin_solution<R>(&self, solution: &mut WorkingSolution, context: RuinContext<R>)
    where
        R: rand::Rng,
    {
        let k = context
            .rng
            .random_range(self.k_min..=self.k_max)
            .min(solution.non_empty_routes_count());

        let mut ruined_routes = FxHashSet::<RouteIdx>::default();

        let mut seed_job = context.problem.random_job(context.rng);

        while ruined_routes.len() < k {
            let route_to_ruin = solution.route_of_job(seed_job);

            if let Some(route_id) = route_to_ruin {
                if context.rng.random_bool(0.5) {
                    self.ruin_string(solution, context.rng, route_id);
                } else {
                    self.ruin_split_string(solution, context.rng, route_id);
                }

                solution.resync_route(route_id);
                ruined_routes.insert(route_id);
            }

            let nearest_service_of_different_route = context
                .problem
                .nearest_jobs(ActivityId::Service(seed_job))
                .find(|&job_id| {
                    if let Some(route_id) = solution.route_of_activity(job_id) {
                        // TODO: tests intersection, it maybe be too restrictive
                        let intersects = match route_to_ruin {
                            Some(ruined_route) => solution
                                .route(ruined_route)
                                .bbox_intersects(solution.route(route_id)),
                            None => true,
                        };

                        intersects && !ruined_routes.contains(&route_id)
                    } else {
                        false
                    }
                });

            if let Some(service_id) = nearest_service_of_different_route {
                seed_job = service_id.job_id();
            } else {
                // No more services to ruin, break the loop
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::test_utils::{self, TestRoute};

    use super::*;

    #[test]
    fn test_compute_possible_string_start() {
        assert_eq!(
            RuinString::compute_possible_string_start(3, 1, 5),
            vec![0, 1]
        );

        assert_eq!(RuinString::compute_possible_string_start(3, 4, 5), vec![2]);
    }

    #[test]
    fn test_remove_split_string() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            }],
        );

        let ruin_string = RuinString::default();
        ruin_string.remove_split_string(
            &mut solution,
            RuinSplitStringParams {
                route_id: RouteIdx::new(0),
                start: 1,
                start_of_preserved_string: 2,
                string_length: 3,
                preserved_string_length: 2,
            },
        );

        assert_eq!(
            solution.route(RouteIdx::new(0)).activity_ids().to_vec(),
            vec![
                ActivityId::service(0),
                ActivityId::service(3),
                ActivityId::service(4),
                ActivityId::service(6),
                ActivityId::service(7),
                ActivityId::service(8)
            ]
        )
    }

    #[test]
    fn test_remove_split_string_2() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            }],
        );

        let ruin_string = RuinString::default();
        ruin_string.remove_split_string(
            &mut solution,
            RuinSplitStringParams {
                route_id: RouteIdx::new(0),
                start: 1,
                start_of_preserved_string: 2,
                string_length: 5,
                preserved_string_length: 2,
            },
        );

        assert_eq!(
            solution.route(RouteIdx::new(0)).activity_ids().to_vec(),
            vec![
                ActivityId::service(0),
                ActivityId::service(3),
                ActivityId::service(4),
                ActivityId::service(8)
            ]
        )
    }
}
