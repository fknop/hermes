use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        intensify::intensify_operator::IntensifyOp, solution::working_solution::WorkingSolution,
    },
};

/// **Inter-Route 2-Opt* (Two-Opt Star)**
///
/// Exchanges the **tails** (remaining activities) of two different routes.
///
/// This operator is designed to fix "crossing" routes. If Route 1 and Route 2
/// cross over each other in an 'X' shape, this operator cuts the 'X' at the intersection
/// and reconnects the routes to be parallel, swapping their destinations.
///
/// # Mechanism
/// 1. **Cut R1** after activity `first_from`.
///    - `R1_Head` = Start ... `first_from`
///    - `R1_Tail` = `first_from_next` ... End
/// 2. **Cut R2** after activity `second_from`.
///    - `R2_Head` = Start ... `second_from`
///    - `R2_Tail` = `second_from_next` ... End
/// 3. **Swap:** Connect `R1_Head` -> `R2_Tail` and `R2_Head` -> `R1_Tail`.
///
/// ```text
/// BEFORE (Routes Cross):
///    R1: [Head A] --x--> [Tail A]
///                    \ /
///                     X  <-- Crossing point
///                    / \
///    R2: [Head B] --x--> [Tail B]
///
/// AFTER (Routes Uncrossed):
///    R1: [Head A] -----> [Tail B]  (New Combination)
///
///    R2: [Head B] -----> [Tail A]  (New Combination)
/// ```
///
/// **Note:** Unlike standard 2-Opt, this usually **preserves the direction** of the tails
/// (i.e., it does not reverse the order of activities within the tail).
#[derive(Debug)]
pub struct InterTwoOptStarOperator {
    params: InterTwoOptStarOperatorParams,
}

#[derive(Debug)]
pub struct InterTwoOptStarOperatorParams {
    pub first_route_id: usize,
    pub second_route_id: usize,
    pub first_from: usize,
    pub second_from: usize,
}

impl InterTwoOptStarOperator {
    pub fn new(params: InterTwoOptStarOperatorParams) -> Self {
        if params.first_route_id == params.second_route_id {
            panic!("InterTwoOptStarOperator cannot be used for intra-route 2-Opt*");
        }

        Self { params }
    }

    pub fn first_route_head<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.first_route_id);
        route.job_ids_iter(0, self.params.first_from + 1)
    }

    pub fn first_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.first_route_id);
        route.job_ids_iter(self.params.first_from + 1, route.len())
    }

    pub fn second_route_head<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.job_ids_iter(0, self.params.second_from + 1)
    }

    pub fn second_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.job_ids_iter(self.params.second_from + 1, route.len())
    }
}

impl IntensifyOp for InterTwoOptStarOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);
        let first_from = r1.location_id(problem, self.params.first_from);
        let first_from_next = r1.next_location_id(problem, self.params.first_from);

        let second_from = r2.location_id(problem, self.params.second_from);
        let second_from_next = r2.next_location_id(problem, self.params.second_from);

        let mut delta = 0.0;

        // Remove edges: (first_from -> first_from_next), (second_from -> second_from_next)
        delta -= problem.travel_cost_or_zero(r1.vehicle(problem), first_from, first_from_next);
        delta -= problem.travel_cost_or_zero(r2.vehicle(problem), second_from, second_from_next);

        // Add edges: (first_from -> second_from_next), (second_from -> first_from_next)
        delta += problem.travel_cost_or_zero(r1.vehicle(problem), first_from, second_from_next);
        delta += problem.travel_cost_or_zero(r2.vehicle(problem), second_from, first_from_next);

        delta
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let r1_head = self.first_route_head(solution);
        let r1_tail = self.first_route_tail(solution);
        let r2_head = self.second_route_head(solution);
        let r2_tail = self.second_route_tail(solution);

        let new_r1_jobs = r1_head.chain(r2_tail);
        let new_r2_jobs = r2_head.chain(r1_tail);

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        r1.is_valid_change(solution.problem(), new_r1_jobs, 0, r1.len())
            && r2.is_valid_change(solution.problem(), new_r2_jobs, 0, r2.len())
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1_head = self.first_route_head(solution);
        let r1_tail = self.first_route_tail(solution);
        let r2_head = self.second_route_head(solution);
        let r2_tail = self.second_route_tail(solution);

        let new_r1_jobs = r1_head.chain(r2_tail).collect::<Vec<_>>();
        let new_r2_jobs = r2_head.chain(r1_tail).collect::<Vec<_>>();

        let r1 = solution.route_mut(self.params.first_route_id);
        r1.replace_activities(problem, &new_r1_jobs, 0, r1.len());

        let r2 = solution.route_mut(self.params.second_route_id);
        r2.replace_activities(problem, &new_r2_jobs, 0, r2.len());
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
            inter_two_opt_star::{InterTwoOptStarOperator, InterTwoOptStarOperatorParams},
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_inter_two_opt_star() {
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

        let operator = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
            first_route_id: 0,
            second_route_id: 1,

            first_from: 2,
            second_from: 2,
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
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 9, 10],
        );

        assert_eq!(
            solution
                .route(1)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 3, 4, 5],
        );
    }

    #[test]
    fn test_inter_two_opt_star_2() {
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

        let operator = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
            first_route_id: 0,
            second_route_id: 1,

            first_from: 4,
            second_from: 3,
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
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 10],
        );

        assert_eq!(
            solution
                .route(1)
                .activity_ids()
                .iter()
                .map(|activity| activity.index())
                .collect::<Vec<_>>(),
            vec![6, 7, 8, 9, 5],
        );
    }

    #[test]
    fn test_inter_two_opt_star_last_transport_cost_delta() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let solution = test_utils::create_test_working_solution(
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

        let operator = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
            first_route_id: 0,
            second_route_id: 1,

            first_from: 5,
            second_from: 4,
        });

        let delta = operator.transport_cost_delta(&solution);
        assert_eq!(delta, 0.0);
    }
}
