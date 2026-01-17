use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Cross-Exchange**
///
/// Swaps a sub-sequence of activities from Route 1 with a sub-sequence from Route 2.
///
/// ```text
/// BEFORE:
///    R1: ... (A) -> [f_start ... f_end] -> (B) ...
///    R2: ... (X) -> [s_start ... s_end] -> (Y) ...
///
/// AFTER:
///    R1: ... (A) -> [s_start ... s_end] -> (B) ...
///    R2: ... (X) -> [f_start ... f_end] -> (Y) ...
/// ```
#[derive(Debug)]
pub struct CrossExchangeOperator {
    params: CrossExchangeOperatorParams,
}

#[derive(Debug)]
pub struct CrossExchangeOperatorParams {
    pub first_route_id: RouteIdx,
    pub second_route_id: RouteIdx,
    pub first_start: usize,
    pub first_end: usize,
    pub second_start: usize,
    pub second_end: usize,
}

impl CrossExchangeOperator {
    pub fn new(params: CrossExchangeOperatorParams) -> Self {
        if params.first_route_id == params.second_route_id {
            panic!("CrossExchangeOperator cannot be used for intra-route exchange");
        }

        if params.first_start >= params.first_end {
            panic!("first_start must be less than first_end");
        }

        if params.second_start >= params.second_end {
            panic!("second_start must be less than second_end");
        }

        Self { params }
    }

    fn first_route_moved_jobs<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        solution
            .route(self.params.first_route_id)
            .job_ids_iter(self.params.first_start, self.params.first_end + 1)
    }

    fn second_route_moved_jobs<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        solution
            .route(self.params.second_route_id)
            .job_ids_iter(self.params.second_start, self.params.second_end + 1)
    }
}

impl LocalSearchOperator for CrossExchangeOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        let previous_first_start = r1.previous_location_id(problem, self.params.first_start);
        let first_start = r1.location_id(problem, self.params.first_start);
        let first_end = r1.location_id(problem, self.params.first_end);
        let next_first_end = r1.next_location_id(problem, self.params.first_end);

        let previous_second_start = r2.previous_location_id(problem, self.params.second_start);
        let second_start = r2.location_id(problem, self.params.second_start);
        let second_end = r2.location_id(problem, self.params.second_end);
        let next_second_end = r2.next_location_id(problem, self.params.second_end);

        let mut delta = 0.0;

        // Route 1 cost change
        delta -=
            problem.travel_cost_or_zero(r1.vehicle(problem), previous_first_start, first_start);
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), first_end, next_first_end);
        delta +=
            problem.travel_cost_or_zero(r1.vehicle(problem), previous_first_start, second_start);
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), second_end, next_first_end);

        // Route 2 cost change
        delta -=
            problem.travel_cost_or_zero(r2.vehicle(problem), previous_second_start, second_start);
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), second_end, next_second_end);
        delta +=
            problem.travel_cost_or_zero(r2.vehicle(problem), previous_second_start, first_start);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), first_end, next_second_end);

        delta
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let first_route = solution.route(self.params.first_route_id);
        let second_route = solution.route(self.params.second_route_id);

        first_route.is_valid_change(
            solution.problem(),
            self.second_route_moved_jobs(solution),
            self.params.first_start,
            self.params.first_end + 1,
        ) && second_route.is_valid_change(
            solution.problem(),
            self.first_route_moved_jobs(solution),
            self.params.second_start,
            self.params.second_end + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let first_route_moved_jobs: Vec<ActivityId> =
            self.first_route_moved_jobs(solution).collect();
        let second_route_moved_jobs: Vec<ActivityId> =
            self.second_route_moved_jobs(solution).collect();

        let first_route = solution.route_mut(self.params.first_route_id);
        first_route.replace_activities(
            problem,
            &second_route_moved_jobs,
            self.params.first_start,
            self.params.first_end + 1,
        );

        let second_route = solution.route_mut(self.params.second_route_id);
        second_route.replace_activities(
            problem,
            &first_route_moved_jobs,
            self.params.second_start,
            self.params.second_end + 1,
        );
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.first_route_id, self.params.second_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::ls::{
            cross_exchange::{CrossExchangeOperator, CrossExchangeOperatorParams},
            r#move::LocalSearchOperator,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_cross_exchange() {
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

        let operator = CrossExchangeOperator::new(CrossExchangeOperatorParams {
            first_route_id: 0.into(),
            first_start: 1,
            first_end: 3,

            second_route_id: 1.into(),
            second_start: 1,
            second_end: 2,
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
                .map(|activity_id| activity_id.job_id().get())
                .collect::<Vec<usize>>(),
            vec![0, 7, 8, 4, 5],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|job_id| job_id.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 1, 2, 3, 9, 10],
        );
    }
}
