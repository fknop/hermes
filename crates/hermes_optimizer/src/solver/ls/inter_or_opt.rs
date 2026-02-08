use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Inter-Route Or-Opt**
///
/// Moves a segment (1-3 consecutive nodes) from one route to another.
///
/// ```text
/// BEFORE:
///    Route 1: ... (A) -> [B -> C] -> (D) -> (E) ...
///                        <─ seg ─>
///    Route 2: ... (X) -> (Y) -> (Z) ...
///
/// AFTER:
///    Route 1: ... (A) -> (D) -> (E) ...
///
///    Route 2: ... (X) -> [B -> C] -> (Y) -> (Z) ...
///                        <─ seg ─>
///
/// Effect: Transfers a cluster of stops to a better-suited route.
/// ```
#[derive(Debug)]
pub struct InterOrOptOperator {
    params: InterOrOptParams,
}

#[derive(Debug)]
pub struct InterOrOptParams {
    pub from_route_id: RouteIdx,
    pub to_route_id: RouteIdx,
    pub segment_start: usize,
    pub segment_length: usize,

    /// Second route position
    pub to: usize,
}

impl InterOrOptOperator {
    pub fn new(params: InterOrOptParams) -> Self {
        if params.segment_length < 2 {
            panic!("InterOrOpt: 'count' must be at least 2.");
        }

        InterOrOptOperator { params }
    }
}

impl LocalSearchOperator for InterOrOptOperator {
    fn generate_moves<C>(
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 == r2 {
            return;
        }

        let from_route = solution.route(r1);
        let to_route = solution.route(r2);

        if from_route.is_empty() {
            return;
        }

        for segment_length in 2..=3 {
            if to_route.will_break_maximum_activities(problem, segment_length) {
                continue;
            }

            for from_pos in 0..from_route
                .activity_ids()
                .len()
                .saturating_sub(segment_length - 1)
            {
                // TODO: handle shipments correctly
                // let from_activity_id = from_route.activity_id(from_pos);

                // if from_activity_id.is_shipment() {
                //     continue; // skip shipments for or-opt
                // }

                for to_pos in 0..=to_route.activity_ids().len() {
                    let op = InterOrOptOperator::new(InterOrOptParams {
                        from_route_id: r1,
                        to_route_id: r2,
                        segment_start: from_pos,
                        segment_length,
                        to: to_pos,
                    });

                    consumer(op)
                }
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let from = r1.location_id(problem, self.params.segment_start);
        let a = r1.previous_location_id(problem, self.params.segment_start);
        let c = r1.location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );
        let d = r1.next_location_id(
            problem,
            self.params.segment_start + self.params.segment_length - 1,
        );

        let x = r2.previous_location_id(problem, self.params.to);
        let y = r2
            .location_id(problem, self.params.to)
            .or_else(|| r2.end_location(problem));

        let mut delta = 0.0;

        // R1 changes
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), a, from);
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), c, d);
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), a, d);

        // R2 changes
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), x, from);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), c, y);
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), x, y);

        delta
    }

    fn fixed_route_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let r1_change = if r1.len() == self.params.segment_length {
            -solution.problem().fixed_vehicle_costs()
        } else {
            0.0
        };

        let r2_change = if r2.is_empty() {
            solution.problem().fixed_vehicle_costs()
        } else {
            0.0
        };

        r1_change + r2_change
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        solution.problem().waiting_duration_cost(
            r1.waiting_duration_change_delta(
                solution.problem(),
                [].into_iter(),
                self.params.segment_start,
                self.params.segment_start + self.params.segment_length,
            ) + r2.waiting_duration_change_delta(
                solution.problem(),
                r1.activity_ids_iter(
                    self.params.segment_start,
                    self.params.segment_start + self.params.segment_length,
                ),
                self.params.to,
                self.params.to,
            ),
        )
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        r1.is_valid_change(
            solution.problem(),
            [].into_iter(),
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        ) && r2.is_valid_change(
            solution.problem(),
            r1.activity_ids_iter(
                self.params.segment_start,
                self.params.segment_start + self.params.segment_length,
            ),
            self.params.to,
            self.params.to,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1 = solution.route_mut(self.params.from_route_id);

        let moved_jobs = r1
            .activity_ids_iter(
                self.params.segment_start,
                self.params.segment_start + self.params.segment_length,
            )
            .collect::<Vec<_>>();

        r1.replace_activities(
            problem,
            &[],
            self.params.segment_start,
            self.params.segment_start + self.params.segment_length,
        );

        let r2 = solution.route_mut(self.params.to_route_id);
        r2.replace_activities(problem, &moved_jobs, self.params.to, self.params.to);
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
                inter_or_opt::{InterOrOptOperator, InterOrOptParams},
                r#move::LocalSearchOperator,
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_inter_or_opt() {
        let locations = test_utils::create_location_grid(6, 7);

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
        let operator = InterOrOptOperator::new(InterOrOptParams {
            from_route_id: RouteIdx::new(0),
            to_route_id: RouteIdx::new(1),
            segment_start: 1,
            segment_length: 3,
            to: 2,
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 4, 5, 6, 7],
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 9, 1, 2, 3, 10],
        );
    }

    #[test]
    fn test_inter_or_opt_end_of_route() {
        let locations = test_utils::create_location_grid(5, 8);

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
        let operator = InterOrOptOperator::new(InterOrOptParams {
            from_route_id: RouteIdx::new(0),
            to_route_id: RouteIdx::new(1),
            segment_start: 1,
            segment_length: 2,
            to: 3,
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 3, 4, 5, 6, 7]
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 9, 10, 1, 2],
        );
    }

    #[test]
    fn test_inter_or_opt_end_of_route_with_return() {
        let locations = test_utils::create_location_grid(5, 8);

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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = InterOrOptOperator::new(InterOrOptParams {
            from_route_id: RouteIdx::new(0),
            to_route_id: RouteIdx::new(1),
            segment_start: 1,
            segment_length: 2,
            to: 3,
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 3, 4, 5, 6, 7]
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 9, 10, 1, 2],
        );
    }

    #[test]
    fn test_inter_or_opt_start_of_route() {
        let locations = test_utils::create_location_grid(5, 5);

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
        let operator = InterOrOptOperator::new(InterOrOptParams {
            from_route_id: RouteIdx::new(0),
            to_route_id: RouteIdx::new(1),
            segment_start: 1,
            segment_length: 2,
            to: 0,
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 3, 4, 5, 6, 7]
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![1, 2, 8, 9, 10],
        );
    }

    #[test]
    fn test_inter_or_opt_end_to_end() {
        let locations = test_utils::create_location_grid(5, 5);

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
        let operator = InterOrOptOperator::new(InterOrOptParams {
            from_route_id: RouteIdx::new(0),
            to_route_id: RouteIdx::new(1),
            segment_start: 6,
            segment_length: 2,
            to: 3,
        });

        let distance0 = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).transport_costs(&problem);
        let delta = operator.delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem)
                + solution.route(RouteIdx::new(1)).transport_costs(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 9, 10, 6, 7],
        );
    }
}
