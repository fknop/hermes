use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
    },
};

/// **Intra-Route Swap**
///
/// Exchanges the positions of two activities (`first` and `second`) within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> [first] -> (B) ... (X) -> [second] -> (Y) ...
///
/// AFTER:
///    ... (A) -> [second] -> (B) ... (X) -> [first] -> (Y) ...
///
/// Note: Handles cases where 'first' and 'second' are adjacent or distant.
/// ```
#[derive(Debug)]
pub struct SwapOperator {
    params: SwapOperatorParams,
}

#[derive(Debug)]
pub struct SwapOperatorParams {
    pub route_id: RouteIdx,
    pub first: usize,
    pub second: usize,
}

impl SwapOperator {
    pub fn new(params: SwapOperatorParams) -> Self {
        if params.first == params.second {
            panic!("SwapOperator: 'first' and 'second' positions must be different.");
        }

        SwapOperator { params }
    }

    /// Returns job IDs [to, ...(from, to), from]
    fn moved_jobs<'a>(
        &'a self,
        route: &'a WorkingSolutionRoute,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        if self.params.first < self.params.second {
            std::iter::once(route.activity_id(self.params.second))
                .chain(route.activity_ids_iter(self.params.first + 1, self.params.second))
                .chain(std::iter::once(route.activity_id(self.params.first)))
        } else {
            std::iter::once(route.activity_id(self.params.first))
                .chain(route.activity_ids_iter(self.params.second + 1, self.params.first))
                .chain(std::iter::once(route.activity_id(self.params.second)))
        }
    }
}

impl LocalSearchOperator for SwapOperator {
    fn generate_moves<C>(
        _problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 != r2 {
            return;
        }

        let route = solution.route(r1);

        for from_pos in 0..route.activity_ids().len() {
            for to_pos in from_pos + 1..route.activity_ids().len() {
                let op = SwapOperator::new(SwapOperatorParams {
                    route_id: r1,
                    first: from_pos,
                    second: to_pos,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let (first, second) = if self.params.first < self.params.second {
            (self.params.first, self.params.second)
        } else {
            (self.params.second, self.params.first)
        };

        let prev_first_loc = route.previous_location_id(problem, first);
        let first_loc = route.location_id(problem, first);
        let next_first_loc = route.next_location_id(problem, first);

        let prev_second_loc = route.previous_location_id(problem, second);
        let second_loc = route.location_id(problem, second);
        let next_second_loc = route.next_location_id(problem, second);

        if second == first + 1 {
            let current_cost =
                problem.travel_cost_or_zero(route.vehicle(problem), prev_first_loc, first_loc)
                    + problem.travel_cost_or_zero(route.vehicle(problem), first_loc, second_loc)
                    + problem.travel_cost_or_zero(
                        route.vehicle(problem),
                        second_loc,
                        next_second_loc,
                    );

            let new_cost =
                problem.travel_cost_or_zero(route.vehicle(problem), prev_first_loc, second_loc)
                    + problem.travel_cost_or_zero(route.vehicle(problem), second_loc, first_loc)
                    + problem.travel_cost_or_zero(
                        route.vehicle(problem),
                        first_loc,
                        next_second_loc,
                    );

            return new_cost - current_cost;
        }

        let current_cost =
            problem.travel_cost_or_zero(route.vehicle(problem), prev_first_loc, first_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), first_loc, next_first_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), prev_second_loc, second_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), second_loc, next_second_loc);

        let new_cost =
            problem.travel_cost_or_zero(route.vehicle(problem), prev_first_loc, second_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), second_loc, next_first_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), prev_second_loc, first_loc)
                + problem.travel_cost_or_zero(route.vehicle(problem), first_loc, next_second_loc);

        new_cost - current_cost
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let route = solution.route(self.params.route_id);
        let moved_jobs = self.moved_jobs(route);

        let delta = route.waiting_duration_change_delta(
            solution.problem(),
            moved_jobs,
            self.params.first.min(self.params.second),
            self.params.first.max(self.params.second) + 1,
        );

        solution.problem().waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);
        let moved_jobs = self.moved_jobs(route);

        route.is_valid_change(
            solution.problem(),
            moved_jobs,
            self.params.first.min(self.params.second),
            self.params.first.max(self.params.second) + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);
        let moved_jobs: Vec<ActivityId> = self.moved_jobs(route).collect();

        solution.route_mut(self.params.route_id).replace_activities(
            problem,
            &moved_jobs,
            self.params.first.min(self.params.second),
            self.params.first.max(self.params.second) + 1,
        );
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
                r#move::LocalSearchOperator,
                swap::{SwapOperator, SwapOperatorParams},
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_swap_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: RouteIdx::new(0),
            first: 1,
            second: 5,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 5, 2, 3, 4, 1],
        );
    }

    #[test]
    fn test_swap_second_before_first_apply() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![0, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9, 10],
                },
            ],
        );

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: RouteIdx::new(0),
            first: 5,
            second: 2,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 5, 3, 4, 2],
        );
    }

    #[test]
    fn test_swap_end_of_route() {
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
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: RouteIdx::new(0),
            first: 0,
            second: 5,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![5, 1, 2, 3, 4, 0],
        );
    }

    #[test]
    fn test_swap_end_of_route_with_return() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0]);
        vehicles[0].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: RouteIdx::new(0),
            first: 0,
            second: 5,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![5, 1, 2, 3, 4, 0],
        );
    }

    #[test]
    fn test_swap_delta() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: RouteIdx::new(0),
            first: 1,
            second: 2,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(delta, 2.0);
    }
}
