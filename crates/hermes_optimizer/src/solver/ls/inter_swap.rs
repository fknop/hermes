use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{Insertion, ServiceInsertion},
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Inter-Route Swap**
///
/// Exchanges an activity `first` in `first_route_id` with `second` in `second_route_id`.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [first] -> (B) ...
///    R2: ... (X) -> [second] -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> [second] -> (B) ...
///    R2: ... (X) -> [first] -> (Y) ...
/// ```
#[derive(Debug)]
pub struct InterSwapOperator {
    params: InterSwapOperatorParams,
}

#[derive(Debug)]
pub struct InterSwapOperatorParams {
    pub first_route_id: RouteIdx,
    pub second_route_id: RouteIdx,
    pub first: usize,
    pub second: usize,
}

impl InterSwapOperator {
    pub fn new(params: InterSwapOperatorParams) -> Self {
        if params.first_route_id == params.second_route_id {
            panic!("InterSwapOperator requires two different route IDs.");
        }

        InterSwapOperator { params }
    }
}

impl LocalSearchOperator for InterSwapOperator {
    fn generate_moves<C>(
        _problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 <= r2 {
            return;
        }

        let from_route = solution.route(r1);
        let to_route = solution.route(r2);

        for from_pos in 0..from_route.activity_ids().len() {
            for to_pos in 0..to_route.activity_ids().len() {
                let op = InterSwapOperator::new(InterSwapOperatorParams {
                    first_route_id: r1,
                    second_route_id: r2,
                    first: from_pos,
                    second: to_pos,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        let first = r1.location_id(problem, self.params.first);
        let second = r2.location_id(problem, self.params.second);

        let a = r1.previous_location_id(problem, self.params.first);
        let b = r1.next_location_id(problem, self.params.first);

        let x = r2.previous_location_id(problem, self.params.second);
        let y = r2.next_location_id(problem, self.params.second);

        let mut delta = 0.0;

        // Route 1 cost change
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), a, first);
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), first, b);
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), a, second);
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), second, b);

        // Route 2 cost change
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), x, second);
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), second, y);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), x, first);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), first, y);

        delta
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let first_route = solution.route(self.params.first_route_id);
        let second_route = solution.route(self.params.second_route_id);

        let first_route_job =
            first_route.activity_ids_iter(self.params.first, self.params.first + 1);
        let second_route_job =
            second_route.activity_ids_iter(self.params.second, self.params.second + 1);

        solution.problem().waiting_duration_cost(
            first_route.waiting_duration_change_delta(
                solution.problem(),
                second_route_job,
                self.params.first,
                self.params.first + 1,
            ) + second_route.waiting_duration_change_delta(
                solution.problem(),
                first_route_job,
                self.params.second,
                self.params.second + 1,
            ),
        )
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let first_route = solution.route(self.params.first_route_id);
        let second_route = solution.route(self.params.second_route_id);

        let first_route_job =
            first_route.activity_ids_iter(self.params.first, self.params.first + 1);
        let second_route_job =
            second_route.activity_ids_iter(self.params.second, self.params.second + 1);

        first_route.is_valid_change(
            solution.problem(),
            second_route_job,
            self.params.first,
            self.params.first + 1,
        ) && second_route.is_valid_change(
            solution.problem(),
            first_route_job,
            self.params.second,
            self.params.second + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        if let Some(first_job_id) = solution
            .route_mut(self.params.first_route_id)
            .remove(problem, self.params.first)
            && let Some(second_job_id) = solution
                .route_mut(self.params.second_route_id)
                .remove(problem, self.params.second)
        {
            solution.route_mut(self.params.first_route_id).insert(
                problem,
                &Insertion::Service(ServiceInsertion {
                    route_id: self.params.first_route_id,
                    position: self.params.first,
                    job_index: second_job_id.job_id(),
                }),
            );
            solution.route_mut(self.params.second_route_id).insert(
                problem,
                &Insertion::Service(ServiceInsertion {
                    route_id: self.params.second_route_id,
                    position: self.params.second,
                    job_index: first_job_id.job_id(),
                }),
            );
        }
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.first_route_id, self.params.second_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::ls::{
            inter_swap::{InterSwapOperator, InterSwapOperatorParams},
            r#move::LocalSearchOperator,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_inter_swap_apply() {
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

        let operator = InterSwapOperator::new(InterSwapOperatorParams {
            first_route_id: 0.into(),
            second_route_id: 1.into(),
            first: 1,
            second: 3,
        });

        let distances = solution.route(0.into()).distance(&problem)
            + solution.route(1.into()).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).distance(&problem)
                + solution.route(1.into()).distance(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 9, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 1, 10],
        );
    }

    #[test]
    fn test_inter_swap_second_before_first() {
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

        let operator = InterSwapOperator::new(InterSwapOperatorParams {
            first_route_id: 0.into(),
            second_route_id: 1.into(),
            first: 4,
            second: 1,
        });

        let distances = solution.route(0.into()).distance(&problem)
            + solution.route(1.into()).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).distance(&problem)
                + solution.route(1.into()).distance(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 7, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 4, 8, 9, 10],
        );
    }

    #[test]
    fn test_inter_swap_end_of_route() {
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

        let operator = InterSwapOperator::new(InterSwapOperatorParams {
            first_route_id: 0.into(),
            second_route_id: 1.into(),
            first: 5,
            second: 4,
        });

        let distances = solution.route(0.into()).distance(&problem)
            + solution.route(1.into()).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).distance(&problem)
                + solution.route(1.into()).distance(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 10],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 5],
        );
    }

    #[test]
    fn test_inter_swap_end_of_route_with_return() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
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

        let operator = InterSwapOperator::new(InterSwapOperatorParams {
            first_route_id: 0.into(),
            second_route_id: 1.into(),
            first: 5,
            second: 4,
        });

        let distances = solution.route(0.into()).distance(&problem)
            + solution.route(1.into()).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).distance(&problem)
                + solution.route(1.into()).distance(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 10],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 5],
        );
    }
}
