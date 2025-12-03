use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Intra-Route 2-Opt**
///
/// Reverses the sequence of activities between `from` and `to` (inclusive).
/// This eliminates crossing edges within a single route.
///
/// ```text
/// BEFORE:
///    ... (prev) --x--> [from] -> ... -> [to] --x--> (next) ...
///          ^             ^               ^            ^
///          A             B               C            D
///
/// AFTER (Sequence Reversed):
///    ... (prev) -----> [to] -> ... -> [from] -----> (next) ...
///          ^             ^               ^            ^
///          A             C               B            D
///
/// Edges Removed: (prev->from), (to->next)
/// Edges Added:   (prev->to),   (from->next)
/// ```
#[derive(Debug)]
pub struct TwoOptOperator {
    params: TwoOptParams,
}

#[derive(Debug)]
pub struct TwoOptParams {
    pub route_id: usize,
    pub from: usize,
    pub to: usize,
}

impl TwoOptOperator {
    pub fn new(params: TwoOptParams) -> Self {
        TwoOptOperator { params }
    }
}

impl TwoOptOperator {
    fn symmetric_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let prev_from = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);

        let to = route.location_id(problem, self.params.to);
        let next_to = route.next_location_id(problem, self.params.to);

        let current_cost =
            problem.travel_cost_or_zero(prev_from, from) + problem.travel_cost_or_zero(to, next_to);

        let new_cost =
            problem.travel_cost_or_zero(prev_from, to) + problem.travel_cost_or_zero(from, next_to);

        new_cost - current_cost
    }

    fn asymmetric_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        if self.params.from >= self.params.to {
            panic!("TwoOpt: cannot have from >= to")
        }

        let mut delta = self.symmetric_delta(solution);

        // Chain reversal
        for i in self.params.from..self.params.to {
            let u = route.location_id(problem, i);
            let v = route.location_id(problem, i + 1);

            // Subtract cost of U -> V
            delta -= problem.travel_cost_or_zero(u, v);
            // Add cost of V -> U
            delta += problem.travel_cost_or_zero(v, u);
        }

        delta
    }
}

impl IntensifyOp for TwoOptOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        if solution.problem().is_symmetric() {
            self.symmetric_delta(solution)
        } else {
            self.asymmetric_delta(solution)
        }
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);

        if self.params.to + 1 > route.len() {
            println!("{:?}", self);
        }

        if self.params.from >= route.len() {
            println!("{:?}", self);
        }

        route.is_valid_tw_change(
            solution.problem(),
            route
                .job_ids_iter(self.params.from, self.params.to + 1)
                .rev(),
            self.params.from,
            self.params.to + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);
        let job_ids = route
            .job_ids_iter(self.params.from, self.params.to + 1)
            .rev()
            .collect::<Vec<_>>();
        route.replace_activities(problem, &job_ids, self.params.from, self.params.to + 1);
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
            two_opt::{TwoOptOperator, TwoOptParams},
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_two_opt() {
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

        let operator = TwoOptOperator::new(TwoOptParams {
            route_id: 0,
            from: 1,
            to: 4,
        });

        let delta = operator.transport_cost_delta(&solution);

        assert_eq!(delta, 6.0);

        let distance = solution.route(0).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(solution.route(0).distance(&problem), distance + delta);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 4, 3, 2, 1, 5]
        );
    }

    #[test]
    fn test_two_opt_end_of_route() {
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

        let operator = TwoOptOperator::new(TwoOptParams {
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
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 5, 4, 3, 2, 1]
        );
    }
}
