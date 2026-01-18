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
        if let Some(job_id) = solution
            .route_mut(self.params.from_route_id)
            .remove(problem, self.params.from)
        {
            let route_to = solution.route_mut(self.params.to_route_id);
            route_to.insert(
                problem,
                &Insertion::Service(ServiceInsertion {
                    route_id: self.params.to_route_id,
                    position: self.params.to,
                    job_index: job_id.job_id(),
                }),
            );
        }
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        problem::job::JobIdx,
        solver::{
            insertion::{Insertion, ServiceInsertion},
            ls::{
                inter_relocate::{InterRelocateOperator, InterRelocateParams},
                r#move::LocalSearchOperator,
            },
            solution::{route_id::RouteIdx, working_solution::WorkingSolution},
        },
        test_utils::{self, TestRoute, create_test_problem_from_json_file, data_fixture_path},
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
        let new_distances = solution.route(0.into()).distance(&problem)
            + solution.route(1.into()).distance(&problem);
        let expected = distances + delta;
        assert!(
            (new_distances - expected).abs() < 1e-9,
            "Distance mismatch: expected {}, got {} (diff: {})",
            expected,
            new_distances,
            (new_distances - expected).abs()
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

    #[tokio::test]
    async fn test_valid_bug() {
        let problem = Arc::new(
            create_test_problem_from_json_file(data_fixture_path("inter_relocate_valid_bug")).await,
        );

        let mut solution = WorkingSolution::new(problem.clone());

        let route1 = [
            JobIdx::new(243),
            JobIdx::new(180),
            JobIdx::new(258),
            JobIdx::new(306),
            JobIdx::new(255),
            JobIdx::new(263),
            JobIdx::new(91),
            JobIdx::new(256),
            JobIdx::new(39),
            JobIdx::new(297),
            JobIdx::new(63),
            JobIdx::new(80),
            JobIdx::new(13),
            JobIdx::new(181),
            JobIdx::new(329),
            JobIdx::new(95),
            JobIdx::new(151),
            JobIdx::new(170),
            JobIdx::new(159),
            JobIdx::new(111),
            JobIdx::new(140),
        ];

        let route13 = [
            JobIdx::new(175),
            JobIdx::new(8),
            JobIdx::new(51),
            JobIdx::new(109),
            JobIdx::new(38),
            JobIdx::new(7),
            JobIdx::new(251),
            JobIdx::new(311),
            JobIdx::new(143),
            JobIdx::new(285),
            JobIdx::new(271),
            JobIdx::new(299),
            JobIdx::new(24),
            JobIdx::new(61),
            JobIdx::new(327),
            JobIdx::new(288),
            JobIdx::new(15),
            JobIdx::new(275),
            JobIdx::new(11),
            JobIdx::new(126),
            JobIdx::new(31),
        ];

        for (index, id) in route1.iter().enumerate() {
            solution.insert(&Insertion::Service(ServiceInsertion {
                job_index: *id,
                position: index,
                route_id: RouteIdx::new(1),
            }));
        }

        for (index, id) in route13.iter().enumerate() {
            solution.insert(&Insertion::Service(ServiceInsertion {
                job_index: *id,
                position: index,
                route_id: RouteIdx::new(13),
            }));
        }

        let op = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: RouteIdx::new(1),
            to_route_id: RouteIdx::new(13),
            from: 11,
            to: 21,
        });

        assert!(!op.is_valid(&solution));

        let mut solution = WorkingSolution::new(problem.clone());
        let route2 = [
            JobIdx::new(56),
            JobIdx::new(166),
            JobIdx::new(131),
            JobIdx::new(270),
            JobIdx::new(308),
            JobIdx::new(89),
            JobIdx::new(229),
            JobIdx::new(41),
            JobIdx::new(322),
            JobIdx::new(139),
            JobIdx::new(296),
            JobIdx::new(310),
            JobIdx::new(220),
            JobIdx::new(85),
            JobIdx::new(119),
            JobIdx::new(156),
            JobIdx::new(28),
            JobIdx::new(231),
            JobIdx::new(92),
            JobIdx::new(86),
        ];
        let route6 = [
            JobIdx::new(254),
            JobIdx::new(245),
            JobIdx::new(70),
            JobIdx::new(173),
            JobIdx::new(10),
            JobIdx::new(6),
            JobIdx::new(120),
            JobIdx::new(280),
            JobIdx::new(272),
            JobIdx::new(104),
            JobIdx::new(123),
            JobIdx::new(65),
            JobIdx::new(234),
            JobIdx::new(331),
            JobIdx::new(199),
            JobIdx::new(29),
            JobIdx::new(129),
            JobIdx::new(210),
            JobIdx::new(117),
            JobIdx::new(20),
            JobIdx::new(309),
            JobIdx::new(34),
            JobIdx::new(286),
            JobIdx::new(287),
            JobIdx::new(300),
        ];

        let op = InterRelocateOperator::new(InterRelocateParams {
            from_route_id: RouteIdx::new(6),
            to_route_id: RouteIdx::new(2),
            from: 0,
            to: 12,
        });

        for (index, id) in route2.iter().enumerate() {
            solution.insert(&Insertion::Service(ServiceInsertion {
                job_index: *id,
                position: index,
                route_id: RouteIdx::new(2),
            }));
        }

        for (index, id) in route6.iter().enumerate() {
            solution.insert(&Insertion::Service(ServiceInsertion {
                job_index: *id,
                position: index,
                route_id: RouteIdx::new(6),
            }));
        }

        assert!(!op.is_valid(&solution));
    }
}

/*
2026-01-18T12:33:31.852464Z ERROR hermes_optimizer::solver::ls::local_search: InterRelocate(InterRelocateOperator { params: InterRelocateParams { from_route_id: RouteIdx(6), to_route_id: RouteIdx(2), from: 0, to: 12 } })
Route 0: [Service(JobIdx(290)), Service(JobIdx(185)), Service(JobIdx(295)), Service(JobIdx(55)), Service(JobIdx(87)), Service(JobIdx(237)), Service(JobIdx(137)), Service(JobIdx(118)), Service(JobIdx(195)), Service(JobIdx(209)), Service(JobIdx(150)), Service(JobIdx(218)), Service(JobIdx(0)), Service(JobIdx(9)), Service(JobIdx(232)), Service(JobIdx(257)), Service(JobIdx(33)), Service(JobIdx(1)), Service(JobIdx(178))]
Route 1: [Service(JobIdx(243)), Service(JobIdx(176)), Service(JobIdx(180)), Service(JobIdx(258)), Service(JobIdx(306)), Service(JobIdx(255)), Service(JobIdx(263)), Service(JobIdx(91)), Service(JobIdx(256)), Service(JobIdx(39)), Service(JobIdx(297)), Service(JobIdx(95)), Service(JobIdx(151)), Service(JobIdx(170)), Service(JobIdx(159)), Service(JobIdx(111)), Service(JobIdx(31)), Service(JobIdx(140)), Service(JobIdx(222))]
Route 2: [Service(JobIdx(56)), Service(JobIdx(166)), Service(JobIdx(131)), Service(JobIdx(270)), Service(JobIdx(308)), Service(JobIdx(89)), Service(JobIdx(229)), Service(JobIdx(41)), Service(JobIdx(322)), Service(JobIdx(139)), Service(JobIdx(296)), Service(JobIdx(310)), Service(JobIdx(254)), Service(JobIdx(220)), Service(JobIdx(85)), Service(JobIdx(119)), Service(JobIdx(156)), Service(JobIdx(28)), Service(JobIdx(231)), Service(JobIdx(92)), Service(JobIdx(86))]
Route 3: [Service(JobIdx(49)), Service(JobIdx(83)), Service(JobIdx(78)), Service(JobIdx(107)), Service(JobIdx(147)), Service(JobIdx(121)), Service(JobIdx(153)), Service(JobIdx(312)), Service(JobIdx(190)), Service(JobIdx(224)), Service(JobIdx(328)), Service(JobIdx(215)), Service(JobIdx(184)), Service(JobIdx(14)), Service(JobIdx(98)), Service(JobIdx(281)), Service(JobIdx(250)), Service(JobIdx(230)), Service(JobIdx(71)), Service(JobIdx(82)), Service(JobIdx(316)), Service(JobIdx(274)), Service(JobIdx(236)), Service(JobIdx(54))]
Route 4: [Service(JobIdx(247)), Service(JobIdx(115)), Service(JobIdx(289)), Service(JobIdx(132)), Service(JobIdx(106)), Service(JobIdx(79)), Service(JobIdx(330)), Service(JobIdx(133)), Service(JobIdx(141)), Service(JobIdx(325)), Service(JobIdx(66)), Service(JobIdx(191)), Service(JobIdx(262)), Service(JobIdx(282)), Service(JobIdx(252)), Service(JobIdx(183)), Service(JobIdx(241)), Service(JobIdx(240)), Service(JobIdx(167)), Service(JobIdx(18)), Service(JobIdx(303))]
Route 5: [Service(JobIdx(277)), Service(JobIdx(242)), Service(JobIdx(174)), Service(JobIdx(103)), Service(JobIdx(172)), Service(JobIdx(239)), Service(JobIdx(246)), Service(JobIdx(46)), Service(JobIdx(203)), Service(JobIdx(145)), Service(JobIdx(171)), Service(JobIdx(100)), Service(JobIdx(273)), Service(JobIdx(283)), Service(JobIdx(53)), Service(JobIdx(127)), Service(JobIdx(154)), Service(JobIdx(212)), Service(JobIdx(214)), Service(JobIdx(221)), Service(JobIdx(37)), Service(JobIdx(160))]
Route 6: [Service(JobIdx(245)), Service(JobIdx(70)), Service(JobIdx(173)), Service(JobIdx(10)), Service(JobIdx(6)), Service(JobIdx(120)), Service(JobIdx(280)), Service(JobIdx(272)), Service(JobIdx(104)), Service(JobIdx(123)), Service(JobIdx(65)), Service(JobIdx(234)), Service(JobIdx(331)), Service(JobIdx(199)), Service(JobIdx(29)), Service(JobIdx(129)), Service(JobIdx(210)), Service(JobIdx(117)), Service(JobIdx(20)), Service(JobIdx(309)), Service(JobIdx(34)), Service(JobIdx(286)), Service(JobIdx(287)), Service(JobIdx(300))]
Route 7: [Service(JobIdx(304)), Service(JobIdx(112)), Service(JobIdx(148)), Service(JobIdx(52)), Service(JobIdx(324)), Service(JobIdx(57)), Service(JobIdx(93)), Service(JobIdx(80)), Service(JobIdx(63)), Service(JobIdx(13)), Service(JobIdx(96)), Service(JobIdx(181)), Service(JobIdx(329)), Service(JobIdx(126)), Service(JobIdx(11)), Service(JobIdx(275)), Service(JobIdx(24)), Service(JobIdx(15)), Service(JobIdx(288)), Service(JobIdx(61)), Service(JobIdx(327))]
Route 8: [Service(JobIdx(48)), Service(JobIdx(32)), Service(JobIdx(158)), Service(JobIdx(198)), Service(JobIdx(124)), Service(JobIdx(144)), Service(JobIdx(69)), Service(JobIdx(168)), Service(JobIdx(223)), Service(JobIdx(68)), Service(JobIdx(45)), Service(JobIdx(47)), Service(JobIdx(228)), Service(JobIdx(207)), Service(JobIdx(30)), Service(JobIdx(292)), Service(JobIdx(4)), Service(JobIdx(206)), Service(JobIdx(317)), Service(JobIdx(16)), Service(JobIdx(169)), Service(JobIdx(84))]
Route 9: [Service(JobIdx(175)), Service(JobIdx(8)), Service(JobIdx(51)), Service(JobIdx(109)), Service(JobIdx(38)), Service(JobIdx(7)), Service(JobIdx(251)), Service(JobIdx(311)), Service(JobIdx(143)), Service(JobIdx(244)), Service(JobIdx(318)), Service(JobIdx(42)), Service(JobIdx(285)), Service(JobIdx(271)), Service(JobIdx(2)), Service(JobIdx(299)), Service(JobIdx(260)), Service(JobIdx(298)), Service(JobIdx(253)), Service(JobIdx(94)), Service(JobIdx(233)), Service(JobIdx(26)), Service(JobIdx(196)), Service(JobIdx(216)), Service(JobIdx(284)), Service(JobIdx(302))]
Route 10: [Service(JobIdx(138)), Service(JobIdx(226)), Service(JobIdx(146)), Service(JobIdx(291)), Service(JobIdx(249)), Service(JobIdx(225)), Service(JobIdx(213)), Service(JobIdx(278)), Service(JobIdx(315)), Service(JobIdx(35)), Service(JobIdx(97)), Service(JobIdx(102)), Service(JobIdx(305)), Service(JobIdx(200)), Service(JobIdx(40)), Service(JobIdx(326)), Service(JobIdx(186)), Service(JobIdx(266)), Service(JobIdx(194)), Service(JobIdx(211)), Service(JobIdx(90)), Service(JobIdx(301)), Service(JobIdx(261)), Service(JobIdx(114)), Service(JobIdx(323))]
Route 11: [Service(JobIdx(165)), Service(JobIdx(219)), Service(JobIdx(27)), Service(JobIdx(320)), Service(JobIdx(74)), Service(JobIdx(267)), Service(JobIdx(128)), Service(JobIdx(276)), Service(JobIdx(204)), Service(JobIdx(164)), Service(JobIdx(136)), Service(JobIdx(259)), Service(JobIdx(238)), Service(JobIdx(149)), Service(JobIdx(182)), Service(JobIdx(279)), Service(JobIdx(77)), Service(JobIdx(5)), Service(JobIdx(162)), Service(JobIdx(72)), Service(JobIdx(155)), Service(JobIdx(319)), Service(JobIdx(73)), Service(JobIdx(201)), Service(JobIdx(134))]
Route 12: [Service(JobIdx(62)), Service(JobIdx(193)), Service(JobIdx(265)), Service(JobIdx(192)), Service(JobIdx(3)), Service(JobIdx(110)), Service(JobIdx(43)), Service(JobIdx(108)), Service(JobIdx(88)), Service(JobIdx(217)), Service(JobIdx(293)), Service(JobIdx(163)), Service(JobIdx(81)), Service(JobIdx(60)), Service(JobIdx(307)), Service(JobIdx(19)), Service(JobIdx(75)), Service(JobIdx(189)), Service(JobIdx(99)), Service(JobIdx(44)), Service(JobIdx(187)), Service(JobIdx(208)), Service(JobIdx(130))]
Route 13: [Service(JobIdx(17)), Service(JobIdx(122)), Service(JobIdx(188)), Service(JobIdx(161)), Service(JobIdx(142)), Service(JobIdx(314)), Service(JobIdx(313)), Service(JobIdx(152)), Service(JobIdx(105)), Service(JobIdx(294)), Service(JobIdx(179)), Service(JobIdx(50)), Service(JobIdx(177)), Service(JobIdx(59)), Service(JobIdx(64)), Service(JobIdx(23)), Service(JobIdx(197)), Service(JobIdx(269)), Service(JobIdx(157)), Service(JobIdx(101)), Service(JobIdx(67)), Service(JobIdx(12)), Service(JobIdx(58)), Service(JobIdx(227)), Service(JobIdx(36)), Service(JobIdx(264))]
 */
