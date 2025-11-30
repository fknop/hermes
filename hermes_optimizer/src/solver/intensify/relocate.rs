use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
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
pub struct RelocateOperator {
    params: RelocateOperatorParams,
}

pub struct RelocateOperatorParams {
    pub route_id: usize,
    pub from: usize,
    pub to: usize,
}

impl RelocateOperator {
    pub fn new(params: RelocateOperatorParams) -> Self {
        if params.from == params.to {
            panic!("RelocateOperator 'from' and 'to' positions must be different");
        }

        Self { params }
    }
}

impl IntensifyOp for RelocateOperator {
    fn delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let route = solution.route(self.params.route_id);

        let prev_from = route.previous_location_id(problem, self.params.from);
        let from = route.location_id(problem, self.params.from);
        let next_from = route.next_location_id(problem, self.params.from);

        let prev_to = if self.params.to < self.params.from {
            route.location_id(problem, self.params.to.wrapping_sub(1))
        } else {
            route.location_id(problem, self.params.to)
        };
        let next_to = route.location_id(problem, self.params.to);

        let current_cost = problem.travel_cost_or_zero(prev_from, from)
            + problem.travel_cost_or_zero(from, next_from)
            + problem.travel_cost_or_zero(prev_to, next_to);
        let new_cost = problem.travel_cost_or_zero(prev_from, next_from)
            + problem.travel_cost_or_zero(prev_to, from)
            + problem.travel_cost_or_zero(from, next_to);

        new_cost - current_cost
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let route = solution.route(self.params.route_id);
        let job_id = route.activities()[self.params.from].job_id();

        // A - B - C - D - E - F
        // Moving B after E, in_between_jobs will be C - D - E
        if self.params.from < self.params.to {
            let in_between_jobs = route.job_ids_iter(self.params.from + 1, self.params.to + 1);

            // Contains C - D - E - B
            let iterator = in_between_jobs.chain(std::iter::once(job_id));
            route.is_valid_tw_change(
                solution.problem(),
                iterator,
                self.params.from,
                self.params.to,
            )
        } else {
            // Moving E before B, in_between_jobs will be E - B - C - D
            let in_between_jobs = route.job_ids_iter(self.params.to, self.params.from);

            // Contains E - B - C - D
            let iterator = std::iter::once(job_id).chain(in_between_jobs);
            route.is_valid_tw_change(
                solution.problem(),
                iterator,
                self.params.to,
                self.params.from + 1,
            )
        }
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let route = solution.route_mut(self.params.route_id);
        route.move_activity(problem, self.params.from, self.params.to);
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
            relocate::{RelocateOperator, RelocateOperatorParams},
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
            route_id: 0,
            from: 1,
            to: 4,
        });

        let delta = operator.delta(&solution);

        assert_eq!(delta, 6.0);

        operator.apply(&problem, &mut solution);

        assert_eq!(
            solution
                .route(0)
                .activities()
                .iter()
                .map(|activity| activity.service_id())
                .collect::<Vec<_>>(),
            vec![0, 2, 3, 1, 4, 5]
        );
    }
}
