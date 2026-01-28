use tracing::info;

use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Reverse Two-Opt** (Inter-route)
///
/// Exchanges the **head** of route T with the **tail** of route S,
/// **reversing both exchanged segments**.
///
/// Given cut points `first_position` in route S and `second_position` in route T:
/// - S's new route = S[0..=first_position] ++ reverse(T[0..=second_position])
/// - T's new route = reverse(S[first_position+1..]) ++ T[second_position+1..]
///
/// ```text
/// BEFORE (first_position=2, second_position=2):
/// Route S: [D]─[S0]─[S1]─[S2]─║─[S3]─[S4]─[S5]─[D]
///                         first_position
/// Route T: [D]─[T0]─[T1]─[T2]─║─[T3]─[T4]─[D]
///                         second_position
///
/// AFTER:
/// Route S: [D]─[S0]─[S1]─[S2]─[T2]─[T1]─[T0]─[D]
///              └─ kept ─┘     └─ T head REV ─┘
/// Route T: [D]─[S5]─[S4]─[S3]─[T3]─[T4]─[D]
///              └─ S tail REV ─┘└ kept ┘
/// ```
#[derive(Debug)]
pub struct InterReverseTwoOptOperator {
    params: InterReverseTwoOptOperatorParams,
}

#[derive(Debug)]
pub struct InterReverseTwoOptOperatorParams {
    pub first_route_id: RouteIdx,
    pub second_route_id: RouteIdx,
    pub first_position: usize,
    pub second_position: usize,
}

impl InterReverseTwoOptOperator {
    pub fn new(params: InterReverseTwoOptOperatorParams) -> Self {
        if params.first_route_id == params.second_route_id {
            panic!("InterReverseTwoOptOperator must have different route IDs");
        }

        Self { params }
    }

    pub fn first_route_head<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.first_route_id);
        route.activity_ids_iter(0, self.params.first_position + 1)
    }

    pub fn first_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.first_route_id);
        route.activity_ids_iter(self.params.first_position + 1, route.len())
    }

    pub fn second_route_head<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.activity_ids_iter(0, self.params.second_position + 1)
    }

    pub fn second_route_tail<'a>(
        &self,
        solution: &'a WorkingSolution,
    ) -> impl DoubleEndedIterator<Item = ActivityId> + Clone + 'a {
        let route = solution.route(self.params.second_route_id);
        route.activity_ids_iter(self.params.second_position + 1, route.len())
    }
}

impl LocalSearchOperator for InterReverseTwoOptOperator {
    fn generate_moves<C>(
        problem: &VehicleRoutingProblem,
        solution: &WorkingSolution,
        (r1, r2): (RouteIdx, RouteIdx),
        mut consumer: C,
    ) where
        C: FnMut(Self),
    {
        if r1 == r2 {
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
                let to_head_length = to_pos + 1;

                if from_route
                    .will_break_maximum_activities(problem, to_head_length - from_tail_length)
                {
                    continue;
                }

                if to_route
                    .will_break_maximum_activities(problem, from_tail_length - to_head_length)
                {
                    continue;
                }

                let op = InterReverseTwoOptOperator::new(InterReverseTwoOptOperatorParams {
                    first_route_id: r1,
                    second_route_id: r2,
                    first_position: from_pos,
                    second_position: to_pos,
                });

                consumer(op)
            }
        }
    }

    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let problem = solution.problem();
        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        r1.transport_cost_delta_update(
            problem,
            self.params.first_position + 1,
            r1.len(),
            r2,
            0,
            self.params.second_position + 1,
        )
        .1 + r2
            .transport_cost_delta_update(
                problem,
                0,
                self.params.second_position + 1,
                r1,
                self.params.first_position + 1,
                r1.len(),
            )
            .1
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        let r1_tail = self.first_route_tail(solution);
        let r2_head = self.second_route_head(solution);

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        let delta = r1.waiting_duration_change_delta(
            solution.problem(),
            r2_head.rev(),
            self.params.first_position + 1,
            r1.len(),
        ) + r2.waiting_duration_change_delta(
            solution.problem(),
            r1_tail.rev(),
            0,
            self.params.second_position + 1,
        );

        solution.problem().waiting_duration_cost(delta)
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        let r1_tail = self.first_route_tail(solution);
        let r2_head = self.second_route_head(solution);

        let r1 = solution.route(self.params.first_route_id);
        let r2 = solution.route(self.params.second_route_id);

        r1.is_valid_change(
            solution.problem(),
            r2_head.rev(),
            self.params.first_position + 1,
            r1.len(),
        ) && r2.is_valid_change(
            solution.problem(),
            r1_tail.rev(),
            0,
            self.params.second_position + 1,
        )
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        let r1_tail = self.first_route_tail(solution).rev().collect::<Vec<_>>();
        let r2_head = self.second_route_head(solution).rev().collect::<Vec<_>>();

        let r1 = solution.route_mut(self.params.first_route_id);
        r1.replace_activities(problem, &r2_head, self.params.first_position + 1, r1.len());

        let r2 = solution.route_mut(self.params.second_route_id);
        r2.replace_activities(problem, &r1_tail, 0, self.params.second_position + 1);
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
            inter_reverse_two_opt::{InterReverseTwoOptOperator, InterReverseTwoOptOperatorParams},
            r#move::LocalSearchOperator,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_reverse_two_opt() {
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

        let operator = InterReverseTwoOptOperator::new(InterReverseTwoOptOperatorParams {
            first_route_id: 0.into(),
            second_route_id: 1.into(),

            first_position: 2,
            second_position: 2,
        });

        let distances = solution.route(0.into()).transport_costs(&problem)
            + solution.route(1.into()).transport_costs(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(0.into()).transport_costs(&problem)
                + solution.route(1.into()).transport_costs(&problem),
            distances + delta,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 8, 7, 6],
        );

        assert_eq!(
            solution
                .route(1.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![5, 4, 3, 9, 10],
        );
    }
}
