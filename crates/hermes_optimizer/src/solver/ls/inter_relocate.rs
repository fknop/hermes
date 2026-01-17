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

        let source_job_id = source_route.job_id(self.params.from);

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
            .remove(self.params.from)
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

    // #[test]
    // fn test_inter_relocate_solomon_c204() {
    //     let current_dir = env::current_dir().unwrap();
    //     let root_directory = current_dir.parent().unwrap();

    //     let path = root_directory.join("./data/solomon/c2/c204.txt");

    //     let problem = Arc::new(SolomonParser::from_file(path.to_str().unwrap()).unwrap());

    //     let route1 = [
    //         4, 74, 1, 0, 98, 99, 96, 91, 93, 94, 97, 6, 2, 3, 88, 90, 89, 87, 83, 85, 82, 81, 84,
    //         75, 70, 69, 72, 79, 78, 80, 77, 76, 86, 95,
    //     ];
    //     let route2 = [
    //         19, 21, 23, 26, 29, 28, 5, 31, 32, 30, 34, 36, 37, 38, 35, 33, 27, 25, 22, 17, 18, 15,
    //         13, 11, 14, 16, 12, 24, 8, 10, 9, 7, 20, 92,
    //     ];
    //     let route3 = [
    //         66, 62, 61, 73, 71, 60, 63, 65, 68, 67, 64, 48, 54, 53, 52, 55, 57, 59, 58, 56, 39, 43,
    //         45, 40, 41, 44, 50, 49, 51, 46, 42, 47,
    //     ];

    //     let mut solution = test_utils::create_test_working_solution(
    //         Arc::clone(&problem),
    //         vec![
    //             TestRoute {
    //                 vehicle_id: 0,
    //                 service_ids: route1.to_vec(),
    //             },
    //             TestRoute {
    //                 vehicle_id: 1,
    //                 service_ids: route2.to_vec(),
    //             },
    //             TestRoute {
    //                 vehicle_id: 2,
    //                 service_ids: route3.to_vec(),
    //             },
    //         ],
    //     );

    //     let operator = InterRelocateOperator::new(InterRelocateParams {
    //         from_route_id: 1,
    //         to_route_id: 0,
    //         from: 33,
    //         to: 0,
    //     });

    //     assert!(operator.is_valid(&solution));

    //     let distances = solution.route(0).distance(&problem) + solution.route(1).distance(&problem);
    //     let delta = operator.transport_cost_delta(&solution);
    //     operator.apply(&problem, &mut solution);
    //     let new_distances =
    //         solution.route(0).distance(&problem) + solution.route(1).distance(&problem);
    //     let expected = distances + delta;
    //     assert!(
    //         (new_distances - expected).abs() < 1e-9,
    //         "Distance mismatch: expected {}, got {} (diff: {})",
    //         expected,
    //         new_distances,
    //         (new_distances - expected).abs()
    //     );
    // }
}
