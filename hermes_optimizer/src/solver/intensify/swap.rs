use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Intra-Route Swap**
///
/// Exchanges the positions of two activities (`first` and `second`) within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> [first] -> (B) ... (X) -> [second] -> (Y) ...
///
/// AFTER:
///    ... (A) -> [second] -> (B) ... (X) -> [first] -> (Y) ...
///
/// Note: Handles cases where 'first' and 'second' are adjacent or distant.
/// ```
pub struct SwapOperator {
    params: SwapOperatorParams,
}

pub struct SwapOperatorParams {
    pub route_id: usize,
    pub first: usize,
    pub second: usize,
}

impl SwapOperator {
    pub fn new(params: SwapOperatorParams) -> Self {
        if params.first == params.second {
            panic!("SwapOperator: 'first' and 'second' positions must be different.");
        }

        SwapOperator { params }
    }
}

impl IntensifyOp for SwapOperator {
    fn delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let (first, second) = if self.params.first < self.params.second {
            (self.params.first, self.params.second)
        } else {
            (self.params.second, self.params.first)
        };

        let prev_first_loc = route.previous_location_id(problem, first);
        let first_loc = route.location_id(problem, first);
        let next_first_loc = route.next_location_id(problem, first);

        let prev_second_loc = route.previous_location_id(problem, second);
        let second_loc = route.location_id(problem, second);
        let next_second_loc = route.next_location_id(problem, second);

        let current_cost = problem.travel_cost_or_zero(prev_first_loc, first_loc)
            + problem.travel_cost_or_zero(first_loc, next_first_loc)
            + problem.travel_cost_or_zero(prev_second_loc, second_loc)
            + problem.travel_cost_or_zero(second_loc, next_second_loc);

        let new_cost = problem.travel_cost_or_zero(prev_first_loc, second_loc)
            + problem.travel_cost_or_zero(second_loc, next_first_loc)
            + problem.travel_cost_or_zero(prev_second_loc, first_loc)
            + problem.travel_cost_or_zero(first_loc, next_second_loc);

        new_cost - current_cost
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!();
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        solution.route_mut(self.params.route_id).swap_activities(
            problem,
            self.params.first,
            self.params.second,
        );
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
            swap::{SwapOperator, SwapOperatorParams},
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_swap() {
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

        let operator = SwapOperator::new(SwapOperatorParams {
            route_id: 0,
            first: 1,
            second: 5,
        });

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 5, 2, 3, 4, 1],
        );
    }
}
