use crate::{
    problem::{job::ActivityId, vehicle_routing_problem::VehicleRoutingProblem},
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{
            route::WorkingSolutionRoute, route_id::RouteIdx, working_solution::WorkingSolution,
        },
    },
};

/// **Inter-Route Mixed Exchange**
///
/// Exchanges a single node from one route with a segment from another.
///
/// ```text
/// BEFORE:
///    Route 1: ... (A) -> (B) -> (C) -> (D) ...
///                         ^
///                       node
///    Route 2: ... (X) -> [Y -> Z] -> (W) ...
///                        <─ seg ─>
///
/// AFTER:
///    Route 1: ... (A) -> [Y -> Z] -> (C) -> (D) ...
///                        <─ seg ─>
///    Route 2: ... (X) -> (B) -> (W) ...
///                         ^
///
/// Effect: Asymmetric exchange useful when routes have uneven density.
/// ```
#[derive(Debug)]
pub struct InterMixedExchange {
    params: InterMixedExchangeParams,
}

#[derive(Debug)]
pub struct InterMixedExchangeParams {
    pub from_route_id: RouteIdx,
    pub to_route_id: RouteIdx,

    /// Position is in the 'from' route
    pub position: usize,

    /// Segment start is in the 'to' route
    pub segment_start: usize,
    pub segment_length: usize,
}

impl InterMixedExchange {
    pub fn new(params: InterMixedExchangeParams) -> Self {
        if params.segment_length < 2 {
            panic!("MixedExchange: 'segment_length' must be at least 2.");
        }

        InterMixedExchange { params }
    }
}

impl LocalSearchOperator for InterMixedExchange {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        todo!()
    }

    fn fixed_route_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        0.0
    }

    fn waiting_cost_delta(&self, _solution: &WorkingSolution) -> f64 {
        todo!()
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        todo!()
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.from_route_id, self.params.to_route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
                inter_mixed_exchange::{InterMixedExchange, InterMixedExchangeParams},
                mixed_exchange::{MixedExchangeOperator, MixedExchangeParams},
                r#move::LocalSearchOperator,
            },
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_mixed_exchange() {
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
        let operator = InterMixedExchange::new(InterMixedExchangeParams {
            from_route_id: RouteIdx::new(1),
            to_route_id: RouteIdx::new(0),
            position: 1, // 9
            segment_start: 2,
            segment_length: 3, // 2, 3, 4
        });

        let distance0 = solution.route(RouteIdx::new(0)).distance(&problem);
        let distance1 = solution.route(RouteIdx::new(1)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem)
                + solution.route(RouteIdx::new(1)).distance(&problem),
            distance0 + distance1 + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 9, 5, 6, 7],
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(1))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![8, 2, 3, 4, 10],
        );
    }
}
