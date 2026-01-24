use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
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
    pub first_route_id: RouteIdx,
    pub second_route_id: RouteIdx,
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
        route.activity_ids_iter(0, self.params.first_from + 1)
    }

    pub fn first_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.first_route_id);
        route.activity_ids_iter(self.params.first_from + 1, route.len())
    }

    pub fn second_route_head<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.activity_ids_iter(0, self.params.second_from + 1)
    }

    pub fn second_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.activity_ids_iter(self.params.second_from + 1, route.len())
    }
}

impl LocalSearchOperator for InterTwoOptStarOperator {
    fn generate_moves<C>(
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 <= r2 {
            return;
        }

        let from_route = solution.route(r1);
        let to_route = solution.route(r2);

        // If the bbox don't intersects, no need to try exchanges
        if !from_route.bbox_intersects(to_route) {
            return;
        }

        let from_route_length = from_route.activity_ids().len();
        let to_route_length = to_route.activity_ids().len();

        for from_pos in 0..from_route_length - 1 {
            for to_pos in 0..to_route_length - 1 {
                let from_tail_length = from_route_length - from_pos - 1;
                let to_tail_length = to_route_length - to_pos - 1;

                if from_route
                    .will_break_maximum_activities(problem, to_tail_length - from_tail_length)
                {
                    continue;
                }

                if to_route
                    .will_break_maximum_activities(problem, from_tail_length - to_tail_length)
                {
                    continue;
                }

                let op = InterTwoOptStarOperator::new(InterTwoOptStarOperatorParams {
                    first_route_id: r1,
                    second_route_id: r2,

                    first_from: from_pos,
                    second_from: to_pos,
                });

                consumer(op)
            }
        }
    }

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

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1_tail = self.first_route_tail(solution);
        let r2_tail = self.second_route_tail(solution);

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        let delta = r1.waiting_duration_change_delta(
            solution.problem(),
            r2_tail,
            self.params.first_from + 1,
            r1.len(),
        ) + r2.waiting_duration_change_delta(
            solution.problem(),
            r1_tail,
            self.params.second_from + 1,
            r2.len(),
        );

        solution.problem().waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let r1_tail = self.first_route_tail(solution);
        let r2_tail = self.second_route_tail(solution);

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        r1.is_valid_change(
            solution.problem(),
            r2_tail,
            self.params.first_from + 1,
            r1.len(),
        ) && r2.is_valid_change(
            solution.problem(),
            r1_tail,
            self.params.second_from + 1,
            r2.len(),
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1_tail = self.first_route_tail(solution).collect::<Vec<_>>();
        let r2_tail = self.second_route_tail(solution).collect::<Vec<_>>();

        let r1 = solution.route_mut(self.params.first_route_id);
        r1.replace_activities(problem, &r2_tail, self.params.first_from + 1, r1.len());

        let r2 = solution.route_mut(self.params.second_route_id);
        r2.replace_activities(problem, &r1_tail, self.params.second_from + 1, r2.len());
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
            inter_two_opt_star::{InterTwoOptStarOperator, InterTwoOptStarOperatorParams},
            r#move::LocalSearchOperator,
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
            first_route_id: 0.into(),
            second_route_id: 1.into(),

            first_from: 2,
            second_from: 2,
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
            vec![0, 1, 2, 9, 10],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
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
            first_route_id: 0.into(),
            second_route_id: 1.into(),

            first_from: 4,
            second_from: 3,
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
            vec![0, 1, 2, 3, 4, 10],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
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
            first_route_id: 0.into(),
            second_route_id: 1.into(),

            first_from: 5,
            second_from: 4,
        });

        let delta = operator.transport_cost_delta(&solution);
        assert_eq!(delta, 0.0);
    }
}
