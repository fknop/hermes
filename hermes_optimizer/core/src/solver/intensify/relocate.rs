use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Intra-Route Relocate**
///
/// Moves a single activity at `from` to a new position at `to`.
/// The node is inserted *at* index `to` (effectively placing it after the node at `to-1`).
///
/// ```text
/// BEFORE:
///    Route: ... (A) -> [from] -> (C) ... (X) -> (Y) ...
///
/// AFTER:
///    Route: ... (A) -> (C) ... (X) -> [from] -> (Y) ...
///                                      ^
///                               Inserted here
///
/// Edges Modified: (A->from), (from->C), (X->Y)
/// Edges Created:  (A->C),    (X->from), (from->Y)
/// ```
#[derive(Debug)]
pub struct RelocateOperator {
    params: RelocateOperatorParams,
}

#[derive(Debug)]
pub struct RelocateOperatorParams {
    pub route_id: usize,
    pub from: usize,
    pub to: usize,
}

impl RelocateOperator {
    pub fn new(params: RelocateOperatorParams) -> Self {
        if params.from == params.to || params.from + 1 == params.to {
            panic!("RelocateOperator 'from' and 'to' positions must be different");
        }

        Self { params }
    }
}

impl IntensifyOp for RelocateOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let prev_from = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);
        let next_from = route.next_location_id(problem, self.params.from);

        let prev_to = route.previous_location_id(problem, self.params.to);
        let next_to = route.location_id(problem, self.params.to);

        let current_cost = problem.travel_cost_or_zero(prev_from, from)
            + problem.travel_cost_or_zero(from, next_from)
            + problem.travel_cost_or_zero(prev_to, next_to);

        let new_cost = problem.travel_cost_or_zero(prev_from, next_from)
            + problem.travel_cost_or_zero(prev_to, from)
            + problem.travel_cost_or_zero(from, next_to);

        new_cost - current_cost
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);
        let job_id = route.activity_ids()[self.params.from];

        // A - B - C - D - E - F
        // Moving B after E, in_between_jobs will be C - D - E
        if self.params.from < self.params.to {
            let in_between_jobs = route.job_ids_iter(self.params.from + 1, self.params.to);

            // Contains C - D - E - B
            let iterator = in_between_jobs.chain(std::iter::once(job_id));
            route.is_valid_change(
                solution.problem(),
                iterator,
                self.params.from,
                self.params.to,
            )
        } else {
            // Moving E before B, in_between_jobs will be E - B - C - D
            let in_between_jobs = route.job_ids_iter(self.params.to, self.params.from);

            // Contains E - B - C - D
            let iterator = std::iter::once(job_id).chain(in_between_jobs);
            route.is_valid_change(
                solution.problem(),
                iterator,
                self.params.to,
                self.params.from + 1,
            )
        }
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);
        let job_id = route.activity_ids()[self.params.from];

        if self.params.from < self.params.to {
            let in_between_jobs = route.job_ids_iter(self.params.from + 1, self.params.to);

            // Contains C - D - E - B
            let iterator = in_between_jobs.chain(std::iter::once(job_id));

            route.replace_activities(
                problem,
                &iterator.collect::<Vec<_>>(),
                self.params.from,
                self.params.to,
            );
        } else {
            // Moving E before B, in_between_jobs will be E - B - C - D
            let in_between_jobs = route.job_ids_iter(self.params.to, self.params.from);

            // Contains E - B - C - D
            let iterator = std::iter::once(job_id).chain(in_between_jobs);
            route.replace_activities(
                problem,
                &iterator.collect::<Vec<_>>(),
                self.params.to,
                self.params.from + 1,
            );
        }

        // route.move_activity(problem, self.params.from, self.params.to);
    }

    fn updated_routes(&self) -> Vec<usize> {
        vec![self.params.route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::intensify::{
            intensify_operator::IntensifyOp,
            relocate::{RelocateOperator, RelocateOperatorParams},
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_relocate() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: 0,
            from: 1,
            to: 4,
        });

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 1, 4, 5]
        );
    }

    #[test]
    fn test_relocate_first() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: 0,
            from: 0,
            to: 3,
        });

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![1, 2, 0, 3, 4, 5]
        );
    }

    #[test]
    fn test_relocate_end_of_route() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: 0,
            from: 1,
            to: 6,
        });

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 4, 5, 1]
        );
    }

    #[test]
    fn test_relocate_start_of_route() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: 0,
            from: 1,
            to: 0,
        });

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![1, 0, 2, 3, 4, 5]
        );
    }

    #[test]
    fn test_relocate_before_end_of_route() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: 0,
            from: 1,
            to: 5,
        });

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 4, 1, 5]
        );
    }
}
