use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        ls::r#move::LocalSearchOperator,
        solution::{route_id::RouteIdx, working_solution::WorkingSolution},
    },
};

/// **Intra-Route Mixed Exchange**
///
/// Swaps a single node with a segment within the same route.
///
/// ```text
/// BEFORE:
///    ... (A) -> (B) -> (C) -> [D -> E] -> (F) ...
///               ^              segment
///             node
///
/// AFTER:
///    ... (A) -> [D -> E] -> (C) -> (B) -> (F) ...
///               segment            ^
///                                node
///
/// Effect: Allows asymmetric exchanges between single stops and clusters.
/// ```
#[derive(Debug)]
pub struct MixedExchangeOperator {
    params: MixedExchangeParams,
}

#[derive(Debug)]
pub struct MixedExchangeParams {
    pub route_id: RouteIdx,
    pub position: usize,
    pub segment_start: usize,
    pub segment_length: usize,
}

impl MixedExchangeOperator {
    pub fn new(params: MixedExchangeParams) -> Self {
        if params.segment_length < 2 {
            panic!("MixedExchange: 'segment_length' must be at least 2.");
        }

        if params.segment_start < params.position
            && params.segment_start + params.segment_length > params.position
        {
            panic!("MixedExchange: the segment cannot overlap with the single position")
        }

        MixedExchangeOperator { params }
    }
}

impl LocalSearchOperator for MixedExchangeOperator {
    fn transport_cost_delta(&self, solution: &WorkingSolution) -> f64 {
        todo!()
    }

    fn is_valid(&self, solution: &WorkingSolution) -> bool {
        todo!()
    }

    fn apply(&self, problem: &VehicleRoutingProblem, solution: &mut WorkingSolution) {
        todo!()
    }

    fn updated_routes(&self) -> Vec<RouteIdx> {
        vec![self.params.route_id]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        solver::{
            ls::{
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
        let operator = MixedExchangeOperator::new(MixedExchangeParams {
            route_id: RouteIdx::new(0),
            position: 1,
            segment_start: 4,
            segment_length: 3,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 4, 5, 6, 2, 3, 1, 7],
        );
    }

    #[test]
    fn test_mixed_exchange_segment_before_position() {
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
        let operator = MixedExchangeOperator::new(MixedExchangeParams {
            route_id: RouteIdx::new(0),
            position: 5,
            segment_start: 1,
            segment_length: 2,
        });

        let distance = solution.route(RouteIdx::new(0)).distance(&problem);
        let delta = operator.transport_cost_delta(&solution);
        operator.apply(&problem, &mut solution);
        assert_eq!(
            solution.route(RouteIdx::new(0)).distance(&problem),
            distance + delta
        );

        assert_eq!(
            solution
                .route(RouteIdx::new(0))
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 5, 3, 4, 1, 2, 6, 7],
        );
    }
}
