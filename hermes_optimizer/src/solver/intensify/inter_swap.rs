use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
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
pub struct InterSwapOperator {
    params: InterSwapOperatorParams,
}

pub struct InterSwapOperatorParams {
    pub first_route_id: usize,
    pub second_route_id: usize,
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

impl IntensifyOp for InterSwapOperator {
    fn delta(&self, solution: &WorkingSolution) -> f64 {
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
        delta -= problem.travel_cost_or_zero(a, first);
        delta -= problem.travel_cost_or_zero(first, b);
        delta += problem.travel_cost_or_zero(a, second);
        delta += problem.travel_cost_or_zero(second, b);

        // Route 2 cost change
        delta -= problem.travel_cost_or_zero(x, second);
        delta -= problem.travel_cost_or_zero(second, y);
        delta += problem.travel_cost_or_zero(x, first);
        delta += problem.travel_cost_or_zero(first, y);

        delta
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        if let Some(first_service_id) = solution
            .route_mut(self.params.first_route_id)
            .remove_activity(problem, self.params.first)
            && let Some(second_service_id) = solution
                .route_mut(self.params.second_route_id)
                .remove_activity(problem, self.params.second)
        {
            solution
                .route_mut(self.params.first_route_id)
                .insert_service(problem, self.params.first, second_service_id);
            solution
                .route_mut(self.params.second_route_id)
                .insert_service(problem, self.params.second, first_service_id);
        }
    }

    fn updated_routes(&self) -> Vec<usize> {
        vec![self.params.first_route_id, self.params.second_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::intensify::{
            intensify_operator::IntensifyOp,
            inter_swap::{InterSwapOperator, InterSwapOperatorParams},
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
            first_route_id: 0,
            second_route_id: 1,
            first: 1,
            second: 3,
        });

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 9, 2, 3, 4, 5],
        );

        assert_eq!(
            solution
                .route(1)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 1, 10],
        );
    }
}
