use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Intra-Route Or-Opt**
///
/// Moves a consecutive chain of activities of length `count` starting at `from`
/// to a new position `to`.
///
/// ```text
/// BEFORE:
///    ... (A) -> [from -> ... -> end] -> (B) ... (X) -> (Y) ...
///                  ^             ^
///              Start Chain   End Chain
///
/// AFTER:
///    ... (A) -> (B) ... (X) -> [from -> ... -> end] -> (Y) ...
///
/// Effect: Moves a whole cluster of stops to a better location.
/// ```
pub struct OrOptOperator {
    params: OrOptOperatorParams,
}

pub struct OrOptOperatorParams {
    pub route_id: usize,
    pub from: usize,
    pub to: usize,
    pub count: usize,
}

impl OrOptOperator {
    pub fn new(params: OrOptOperatorParams) -> Self {
        if params.count < 2 {
            panic!("OrOptOperator: 'count' must be at least 2.");
        }

        if params.from == params.to {
            panic!("OrOptOperator: 'from' and 'to' positions must be different.");
        }

        if params.from + params.count > params.to && params.to > params.from {
            panic!("OrOptOperator: Overlapping segments are not allowed.");
        }

        if params.to + params.count > params.from && params.from > params.to {
            panic!("OrOptOperator: Overlapping segments are not allowed.");
        }

        OrOptOperator { params }
    }
}

impl IntensifyOp for OrOptOperator {
    fn delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let A = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);

        let end = route.location_id(problem, self.params.from + self.params.count);
        let B = route.next_location_id(problem, self.params.from + self.params.count);

        let X = route.location_id(problem, self.params.to);
        let Y = route.next_location_id(problem, self.params.to);

        let mut delta = 0.0;

        delta -= problem.travel_cost_or_zero(A, from);
        delta -= problem.travel_cost_or_zero(end, B);
        delta -= problem.travel_cost_or_zero(X, Y);
        delta += problem.travel_cost_or_zero(A, B);
        delta += problem.travel_cost_or_zero(X, from);
        delta += problem.travel_cost_or_zero(end, Y);

        delta
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);

        let job_ids = route
            .job_ids_iter(self.params.from, self.params.from + self.params.count)
            .collect::<Vec<_>>();

        if self.params.from < self.params.to {
            // Insert activities at new position
            route.replace_activities(problem, &job_ids, self.params.to, self.params.to);

            // Remove activities at old position
            route.replace_activities(
                problem,
                &[],
                self.params.from,
                self.params.from + self.params.count,
            );
        } else {
            // Remove activities at old position
            route.replace_activities(
                problem,
                &[],
                self.params.from,
                self.params.from + self.params.count,
            );

            // Insert activities at new position
            route.replace_activities(problem, &job_ids, self.params.to, self.params.to);
        }
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
            or_opt::{OrOptOperator, OrOptOperatorParams},
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_or_opt() {
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
                    service_ids: vec![0, 1, 2, 3, 4, 5, 6, 7],
                },
                TestRoute {
                    vehicle_id: 1,
                    service_ids: vec![8, 9, 10],
                },
            ],
        );

        // Move [1, 2, 3] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: 0,
            from: 1,
            count: 3,
            to: 5,
        });

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 4, 1, 2, 3, 5, 6, 7],
        );

        // Move [3, 5] to position after 4
        let operator = OrOptOperator::new(OrOptOperatorParams {
            route_id: 0,
            from: 4,
            count: 2,
            to: 2,
        });

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 4, 3, 5, 1, 2, 6, 7],
        );
    }
}
