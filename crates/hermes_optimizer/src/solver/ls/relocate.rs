use tracing::{Level, instrument};

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
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
    pub route_id: RouteIdx,
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

impl LocalSearchOperator for RelocateOperator {
    #[instrument(skip_all,level = Level::DEBUG)]
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
            let from_id = route.activity_id(from_pos);

            let (to_pos_start, to_pos_end) = match from_id {
                ActivityId::ShipmentPickup(index) => {
                    let delivery_position = route
                        .job_position(ActivityId::ShipmentDelivery(index))
                        .unwrap_or_else(|| {
                            panic!("Shipment pickup {from_id} has no delivery in the same route")
                        });
                    (from_pos + 1, delivery_position)
                }
                ActivityId::ShipmentDelivery(index) => {
                    let pickup_position = route
                        .job_position(ActivityId::ShipmentPickup(index))
                        .unwrap_or_else(|| {
                            panic!("Shipment delivery {from_id} has no pickup in the same route")
                        });
                    (pickup_position + 1, route.len())
                }
                ActivityId::Service(_) => (0, route.len()),
            };

            for to_pos in to_pos_start..=to_pos_end {
                if from_pos == to_pos {
                    continue;
                }

                if from_pos + 1 == to_pos {
                    continue; // no change in this case
                }

                let op = RelocateOperator::new(RelocateOperatorParams {
                    route_id: r1,
                    from: from_pos,
                    to: to_pos,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let a = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);
        let c = route.next_location_id(problem, self.params.from);

        let x = route.previous_location_id(problem, self.params.to);
        let y = route
            .location_id(problem, self.params.to)
            .or_else(|| route.end_location(problem));

        let current_cost = problem.travel_cost_or_zero(route.vehicle(problem), a, from)
            + problem.travel_cost_or_zero(route.vehicle(problem), from, c)
            + problem.travel_cost_or_zero(route.vehicle(problem), x, y);

        let new_cost = problem.travel_cost_or_zero(route.vehicle(problem), a, c)
            + problem.travel_cost_or_zero(route.vehicle(problem), x, from)
            + problem.travel_cost_or_zero(route.vehicle(problem), from, y);

        new_cost - current_cost
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let route = solution.route(self.params.route_id);
        let job_id = route.activity_ids()[self.params.from];

        let delta = if self.params.from < self.params.to {
            let in_between_jobs = route.activity_ids_iter(self.params.from + 1, self.params.to);

            // Contains C - D - E - B
            let iterator = in_between_jobs.chain(std::iter::once(job_id));
            route.waiting_duration_change_delta(
                solution.problem(),
                iterator,
                self.params.from,
                self.params.to,
            )
        } else {
            // Moving E before B, in_between_jobs will be B - C - D
            let in_between_jobs = route.activity_ids_iter(self.params.to, self.params.from);

            // Contains E - B - C - D
            let iterator = std::iter::once(job_id).chain(in_between_jobs);
            route.waiting_duration_change_delta(
                solution.problem(),
                iterator,
                self.params.to,
                self.params.from + 1,
            )
        };

        solution.problem().waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);
        let job_id = route.activity_ids()[self.params.from];

        // A - B - C - D - E - F
        // Moving B after E, in_between_jobs will be C - D - E
        if self.params.from < self.params.to {
            let in_between_jobs = route.activity_ids_iter(self.params.from + 1, self.params.to);

            // Contains C - D - E - B
            let iterator = in_between_jobs.chain(std::iter::once(job_id));
            route.is_valid_change(
                solution.problem(),
                iterator,
                self.params.from,
                self.params.to,
            )
        } else {
            // Moving E before B, in_between_jobs will be B - C - D
            let in_between_jobs = route.activity_ids_iter(self.params.to, self.params.from);

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
            let in_between_jobs = route.activity_ids_iter(self.params.from + 1, self.params.to);

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
            let in_between_jobs = route.activity_ids_iter(self.params.to, self.params.from);

            // Contains E - B - C - D
            let iterator = std::iter::once(job_id).chain(in_between_jobs);
            route.replace_activities(
                problem,
                &iterator.collect::<Vec<_>>(),
                self.params.to,
                self.params.from + 1,
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
                relocate::{RelocateOperator, RelocateOperatorParams},
            },
            solution::route_id::RouteIdx,
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
            route_id: RouteIdx::new(0),
            from: 1,
            to: 4,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
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
            route_id: RouteIdx::new(0),
            from: 0,
            to: 3,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![1, 2, 0, 3, 4, 5]
        );
    }

    #[test]
    fn test_relocate_one_before() {
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
            route_id: RouteIdx::new(0),
            from: 4,
            to: 3,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 4, 3, 5],
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
            route_id: RouteIdx::new(0),
            from: 1,
            to: 6,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 4, 5, 1]
        );
    }

    #[test]
    fn test_relocate_end_of_route_with_return() {
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

        let operator = RelocateOperator::new(RelocateOperatorParams {
            route_id: RouteIdx::new(0),
            from: 1,
            to: 6,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
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
            route_id: RouteIdx::new(0),
            from: 1,
            to: 0,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
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
            route_id: RouteIdx::new(0),
            from: 1,
            to: 5,
        });

        let distance = solution.route(RouteIdx::new(0)).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).transport_costs(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 4, 1, 5]
        );
    }
}
