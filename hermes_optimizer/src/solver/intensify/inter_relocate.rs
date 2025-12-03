use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
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
    pub from_route_id: usize,
    pub to_route_id: usize,
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

impl IntensifyOp for InterRelocateOperator {
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

        delta -= problem.travel_cost_or_zero(a, from);
        delta -= problem.travel_cost_or_zero(from, b);
        delta += problem.travel_cost_or_zero(a, b);

        delta -= problem.travel_cost_or_zero(x, y);
        delta += problem.travel_cost_or_zero(x, from);
        delta += problem.travel_cost_or_zero(from, y);

        delta
    }

    fn fixed_route_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1 = solution.route(self.params.from_route_id);

        if r1.len() == 1 {
            -solution.problem().fixed_vehicle_costs()
        } else {
            0.0
        }
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let source_route = solution.route(self.params.from_route_id);
        let target_route = solution.route(self.params.to_route_id);

        let source_job_id = source_route.job_id_at(self.params.from);

        target_route.is_valid_tw_change(
            solution.problem(),
            std::iter::once(source_job_id),
            self.params.to,
            self.params.to,
        ) && source_route.is_valid_tw_change(
            solution.problem(),
            source_route.job_ids_iter(self.params.from + 1, self.params.from + 1),
            self.params.from,
            self.params.from + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        if let Some(service_id) = solution
            .route_mut(self.params.from_route_id)
            .remove_activity(problem, self.params.from)
        {
            let route_to = solution.route_mut(self.params.to_route_id);
            route_to.insert_service(problem, self.params.to, service_id);
        }
    }

    fn updated_routes(&self) -> Vec<usize> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::intensify::{
            intensify_operator::IntensifyOp,
            inter_relocate::{InterRelocateOperator, InterRelocateParams},
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
            from_route_id: 0,
            to_route_id: 1,
            from: 1,
            to: 4,
        });

        let distances = solution.route(0).distance(&problem) + solution.route(1).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0).distance(&problem) + solution.route(1).distance(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 1, 10],
        );
    }
}
