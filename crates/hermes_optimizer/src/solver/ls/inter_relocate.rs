use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{Insertion, ServiceInsertion},
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Inter-Route Relocate**
///
/// Moves an activity `from` in `from_route_id` to position `to` in `to_route_id`.
/// Crucial for load balancing and route elimination.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [from] -> (B) ...
///    R2: ... (X) -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> (B) ...
///    R2: ... (X) -> [from] -> (Y) ...
/// ```
#[derive(Debug)]
pub struct InterRelocateOperator {
    params: InterRelocateParams,
}

#[derive(Debug)]
pub struct InterRelocateParams {
    pub from_route_id: RouteIdx,
    pub to_route_id: RouteIdx,
    pub from: usize,
    pub to: usize,
}

impl InterRelocateOperator {
    pub fn new(params: InterRelocateParams) -> Self {
        if params.from_route_id == params.to_route_id {
            panic!("InterRelocateOperator cannot be used for intra-route relocation");
        }

        Self { params }
    }
}

impl LocalSearchOperator for InterRelocateOperator {
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

        if to_route.breaks_maximum_activities(problem, 1) {
            return;
        }

        for from_pos in 0..from_route.activity_ids().len() {
            let from_activity_id = from_route.activity_id(from_pos);

            if from_activity_id.is_shipment() {
                continue; // skip shipments for inter-relocate
            }

            for to_pos in 0..=to_route.activity_ids().len() {
                let op = InterRelocateOperator::new(InterRelocateParams {
                    from_route_id: r1,
                    to_route_id: r2,
                    from: from_pos,
                    to: to_pos,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let from = r1.location_id(problem, self.params.from);
        let a = r1.previous_location_id(problem, self.params.from);
        let b = r1.next_location_id(problem, self.params.from);

        let x = r2.previous_location_id(problem, self.params.to);
        let y = r2.location_id(problem, self.params.to);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), a, from);
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), from, b);
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), a, b);

        delta += problem.travel_cost_or_zero(r2.vehicle(problem), x, from);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), from, y);
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), x, y);

        delta
    }

    fn fixed_route_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1 = solution.route(self.params.from_route_id);
        let r2 = solution.route(self.params.to_route_id);

        let r1_change = if r1.len() == 1 {
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
        let source_route = solution.route(self.params.from_route_id);
        let target_route = solution.route(self.params.to_route_id);
        let source_job_id = source_route.activity_id(self.params.from);

        solution.problem().waiting_duration_cost(
            target_route.waiting_duration_change_delta(
                solution.problem(),
                std::iter::once(source_job_id),
                self.params.to,
                self.params.to,
            ) + source_route.waiting_duration_change_delta(
                solution.problem(),
                [].into_iter(),
                self.params.from,
                self.params.from + 1,
            ),
        )
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let source_route = solution.route(self.params.from_route_id);
        let target_route = solution.route(self.params.to_route_id);

        let source_job_id = source_route.activity_id(self.params.from);

        let is_target_route_valid = target_route.is_valid_change(
            solution.problem(),
            std::iter::once(source_job_id),
            self.params.to,
            self.params.to,
        );

        let is_source_route_valid = source_route.is_valid_change(
            solution.problem(),
            [].into_iter(),
            self.params.from,
            self.params.from + 1,
        );

        is_target_route_valid && is_source_route_valid
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let source_job_id = solution
            .route(self.params.from_route_id)
            .activity_id(self.params.from);
        solution
            .route_mut(self.params.from_route_id)
            .replace_activities(problem, &[], self.params.from, self.params.from + 1);

        solution
            .route_mut(self.params.to_route_id)
            .replace_activities(problem, &[source_job_id], self.params.to, self.params.to);
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::ls::{
            inter_relocate::{InterRelocateOperator, InterRelocateParams},
            r#move::LocalSearchOperator,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_inter_relocate() {
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

        let operator = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: 0.into(),
            to_route_id: 1.into(),
            from: 1,
            to: 4,
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
            vec![0, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 1, 10],
        );
    }

    #[test]
    fn test_inter_relocate_first() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![10, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9],
                },
            ],
        );

        let operator = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: 0.into(),
            to_route_id: 1.into(),
            from: 0,
            to: 0,
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
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![10, 6, 7, 8, 9],
        );
    }

    #[test]
    fn test_inter_relocate_end() {
        let locations = test_utils::create_location_grid(5, 5);

        let services = test_utils::create_basic_services(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![10, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9],
                },
            ],
        );

        let operator = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: 0.into(),
            to_route_id: 1.into(),
            from: 1,
            to: 4,
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
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![10, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 1],
        );
    }

    #[test]
    fn test_inter_relocate_with_return() {
        let locations = test_utils::create_location_grid(7, 7);

        let services = test_utils::create_basic_services(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);

        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![
                TestRoute {
                    vehicle_id: 0,
                    service_ids: vec![10, 1, 2, 3, 4, 5],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![6, 7, 8, 9],
                },
            ],
        );

        let operator = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: 0.into(),
            to_route_id: 1.into(),
            from: 0,
            to: 4,
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
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 10],
        );
    }

    //     #[tokio::test]
    //     async fn test_valid_bug() {
    //         let problem = Arc::new(
    //             create_test_problem_from_json_file(data_fixture_path("inter_relocate_valid_bug")).await,
    //         );

    //         let mut solution = WorkingSolution::new(problem.clone());

    //         let route1 = [
    //             JobIdx::new(243),
    //             JobIdx::new(180),
    //             JobIdx::new(258),
    //             JobIdx::new(306),
    //             JobIdx::new(255),
    //             JobIdx::new(263),
    //             JobIdx::new(91),
    //             JobIdx::new(256),
    //             JobIdx::new(39),
    //             JobIdx::new(297),
    //             JobIdx::new(63),
    //             JobIdx::new(80),
    //             JobIdx::new(13),
    //             JobIdx::new(181),
    //             JobIdx::new(329),
    //             JobIdx::new(95),
    //             JobIdx::new(151),
    //             JobIdx::new(170),
    //             JobIdx::new(159),
    //             JobIdx::new(111),
    //             JobIdx::new(140),
    //         ];

    //         let route13 = [
    //             JobIdx::new(175),
    //             JobIdx::new(8),
    //             JobIdx::new(51),
    //             JobIdx::new(109),
    //             JobIdx::new(38),
    //             JobIdx::new(7),
    //             JobIdx::new(251),
    //             JobIdx::new(311),
    //             JobIdx::new(143),
    //             JobIdx::new(285),
    //             JobIdx::new(271),
    //             JobIdx::new(299),
    //             JobIdx::new(24),
    //             JobIdx::new(61),
    //             JobIdx::new(327),
    //             JobIdx::new(288),
    //             JobIdx::new(15),
    //             JobIdx::new(275),
    //             JobIdx::new(11),
    //             JobIdx::new(126),
    //             JobIdx::new(31),
    //         ];

    //         for (index, id) in route1.iter().enumerate() {
    //             solution.insert(&Insertion::Service(ServiceInsertion {
    //                 job_index: *id,
    //                 position: index,
    //                 route_id: RouteIdx::new(1),
    //             }));
    //         }

    //         for (index, id) in route13.iter().enumerate() {
    //             solution.insert(&Insertion::Service(ServiceInsertion {
    //                 job_index: *id,
    //                 position: index,
    //                 route_id: RouteIdx::new(13),
    //             }));
    //         }

    //         let op = InterRelocateOperator::new(InterRelocateParams {
    //             from_route_id: RouteIdx::new(1),
    //             to_route_id: RouteIdx::new(13),
    //             from: 11,
    //             to: 21,
    //         });

    //         assert!(!op.is_valid(&solution));

    //         let mut solution = WorkingSolution::new(problem.clone());
    //         let route2 = [
    //             JobIdx::new(56),
    //             JobIdx::new(166),
    //             JobIdx::new(131),
    //             JobIdx::new(270),
    //             JobIdx::new(308),
    //             JobIdx::new(89),
    //             JobIdx::new(229),
    //             JobIdx::new(41),
    //             JobIdx::new(322),
    //             JobIdx::new(139),
    //             JobIdx::new(296),
    //             JobIdx::new(310),
    //             JobIdx::new(220),
    //             JobIdx::new(85),
    //             JobIdx::new(119),
    //             JobIdx::new(156),
    //             JobIdx::new(28),
    //             JobIdx::new(231),
    //             JobIdx::new(92),
    //             JobIdx::new(86),
    //         ];
    //         let route6 = [
    //             JobIdx::new(254),
    //             JobIdx::new(245),
    //             JobIdx::new(70),
    //             JobIdx::new(173),
    //             JobIdx::new(10),
    //             JobIdx::new(6),
    //             JobIdx::new(120),
    //             JobIdx::new(280),
    //             JobIdx::new(272),
    //             JobIdx::new(104),
    //             JobIdx::new(123),
    //             JobIdx::new(65),
    //             JobIdx::new(234),
    //             JobIdx::new(331),
    //             JobIdx::new(199),
    //             JobIdx::new(29),
    //             JobIdx::new(129),
    //             JobIdx::new(210),
    //             JobIdx::new(117),
    //             JobIdx::new(20),
    //             JobIdx::new(309),
    //             JobIdx::new(34),
    //             JobIdx::new(286),
    //             JobIdx::new(287),
    //             JobIdx::new(300),
    //         ];

    //         let op = InterRelocateOperator::new(InterRelocateParams {
    //             from_route_id: RouteIdx::new(6),
    //             to_route_id: RouteIdx::new(2),
    //             from: 0,
    //             to: 12,
    //         });

    //         for (index, id) in route2.iter().enumerate() {
    //             solution.insert(&Insertion::Service(ServiceInsertion {
    //                 job_index: *id,
    //                 position: index,
    //                 route_id: RouteIdx::new(2),
    //             }));
    //         }

    //         for (index, id) in route6.iter().enumerate() {
    //             solution.insert(&Insertion::Service(ServiceInsertion {
    //                 job_index: *id,
    //                 position: index,
    //                 route_id: RouteIdx::new(6),
    //             }));
    //         }

    //         assert!(!op.is_valid(&solution));
    //     }
}
