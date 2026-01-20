use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
    },
};

/// **Intra-Route Or-Opt**
///
/// Moves a consecutive chain of activities of length `count` starting at `from`
/// to a new position `to`.
///
/// ```text
/// BEFORE:
///    ... (A) -> [from -> ... -> end] -> (B) ... (X) -> (Y) ...
///                  ^             ^
///              Start Chain   End Chain
///
/// AFTER:
///    ... (A) -> (B) ... (X) -> [from -> ... -> end] -> (Y) ...
///
/// Effect: Moves a whole cluster of stops to a better location.
/// ```
#[derive(Debug)]
pub struct OrOptOperator {
    params: OrOptOperatorParams,
}

#[derive(Debug)]
pub struct OrOptOperatorParams {
    pub route_id: RouteIdx,
    pub from: usize,
    pub to: usize,
    pub segment_length: usize,
}

impl OrOptOperator {
    pub fn new(params: OrOptOperatorParams) -> Self {
        if params.segment_length < 2 {
            panic!("OrOptOperator: 'count' must be at least 2.");
        }

        if params.from == params.to {
            panic!(
                "OrOptOperator: 'from' ({}) and 'to' ({}) positions must be different.",
                params.from, params.to
            );
        }

        if params.from + params.segment_length >= params.to && params.to > params.from {
            panic!("OrOptOperator: Overlapping segments are not allowed.");
        }

        OrOptOperator { params }
    }

    fn moved_jobs<'a>(
        &'a self,
        route: &'a WorkingSolutionRoute,
    ) -> impl Iterator<Item = ActivityId> + Clone + 'a {
        if self.params.from < self.params.to {
            let moved_jobs = route.job_ids_iter(
                self.params.from,
                self.params.from + self.params.segment_length,
            );

            let in_between_jobs = route.job_ids_iter(
                self.params.from + self.params.segment_length,
                self.params.to,
            );

            in_between_jobs.chain(moved_jobs)
        } else {
            let moved_jobs = route.job_ids_iter(
                self.params.from,
                self.params.from + self.params.segment_length,
            );

            let in_between_jobs = route.job_ids_iter(self.params.to, self.params.from);

            moved_jobs.chain(in_between_jobs)
        }
    }
}

impl LocalSearchOperator for OrOptOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let a = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);

        let end = route.location_id(problem, self.params.from + self.params.segment_length - 1);
        let b = route.next_location_id(problem, self.params.from + self.params.segment_length - 1);

        let x = route.previous_location_id(problem, self.params.to);

        let y = route.location_id(problem, self.params.to);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(route.vehicle(problem), a, from);
        delta -= problem.travel_cost_or_zero(route.vehicle(problem), end, b);
        delta -= problem.travel_cost_or_zero(route.vehicle(problem), x, y);

        delta += problem.travel_cost_or_zero(route.vehicle(problem), a, b);
        delta += problem.travel_cost_or_zero(route.vehicle(problem), x, from);
        delta += problem.travel_cost_or_zero(route.vehicle(problem), end, y);

        delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        if self.params.from < self.params.to {
            let route = solution.route(self.params.route_id);

            let moved_jobs = self.moved_jobs(route);

            solution
                .problem()
                .waiting_duration_cost(route.waiting_duration_change_delta(
                    solution.problem(),
                    moved_jobs,
                    self.params.from,
                    self.params.to,
                ))
        } else {
            let route = solution.route(self.params.route_id);

            let moved_jobs = self.moved_jobs(route);

            solution
                .problem()
                .waiting_duration_cost(route.waiting_duration_change_delta(
                    solution.problem(),
                    moved_jobs,
                    self.params.to,
                    self.params.from + self.params.segment_length,
                ))
        }
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        if self.params.from < self.params.to {
            let route = solution.route(self.params.route_id);

            let moved_jobs = self.moved_jobs(route);

            route.is_valid_change(
                solution.problem(),
                moved_jobs,
                self.params.from,
                self.params.to,
            )
        } else {
            let route = solution.route(self.params.route_id);

            let moved_jobs = self.moved_jobs(route);

            route.is_valid_change(
                solution.problem(),
                moved_jobs,
                self.params.to,
                self.params.from + self.params.segment_length,
            )
        }
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);

        if self.params.from < self.params.to {
            let job_ids = self.moved_jobs(route).collect::<Vec<_>>();

            // Insert activities at new position
            route.replace_activities(problem, &job_ids, self.params.from, self.params.to);
        } else {
            let job_ids = self.moved_jobs(route).collect::<Vec<_>>();

            // Insert activities at new position
            route.replace_activities(
                problem,
                &job_ids,
                self.params.to,
                self.params.from + self.params.segment_length,
            );
        }
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
                or_opt::{OrOptOperator, OrOptOperatorParams},
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_or_opt() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 1,
            segment_length: 3,
            to: 5,
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
            vec![0, 4, 1, 2, 3, 5, 6, 7],
        );

        // Move [3, 5] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 4,
            segment_length: 2,
            to: 2,
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
            vec![0, 4, 3, 5, 1, 2, 6, 7],
        );
    }

    #[test]
    fn test_or_opt_to_before_from() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 4,
            segment_length: 2,
            to: 1,
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
            vec![0, 4, 5, 1, 2, 3, 6, 7],
        );
    }

    #[test]
    fn test_or_opt_end_of_route() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 1,
            segment_length: 3,
            to: 8,
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
            vec![0, 4, 5, 6, 7, 1, 2, 3],
        );
    }

    #[test]
    fn test_or_opt_delta() {
        let locations = test_utils::create_location_grid(20, 20);

        let services = test_utils::create_basic_services(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            }],
        );

        // Move [0..8] to position after 9
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 0,
            segment_length: 9,
            to: 10,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        assert_eq!(distance, 11.0);

        let delta = operator.transport_cost_delta(&solution);

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );
        assert_eq!(delta, 18.0);

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 10],
        );
    }

    #[test]
    #[should_panic(expected = "OrOptOperator: Overlapping segments are not allowed.")]
    fn test_or_opt_consecutive() {
        OrOptOperator::new(OrOptOperatorParams {
            route_id: RouteIdx::new(0),
            from: 1,
            segment_length: 3,
            to: 4,
        });
    }
}
