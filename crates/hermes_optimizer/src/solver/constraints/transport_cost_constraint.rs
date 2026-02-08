use crate::solver::{
    insertion::Insertion, insertion_context::InsertionContext, score::Score,
    score_level::ScoreLevel, solution::working_solution::WorkingSolution,
};

use super::global_constraint::GlobalConstraint;

#[derive(Clone)]
pub struct TransportCostConstraint;

pub const TRANSPORT_COST_WEIGHT: f64 = 1.0;

const SCORE_LEVEL: ScoreLevel = ScoreLevel::Soft;

impl GlobalConstraint for TransportCostConstraint {
    fn score_level(&self) -> ScoreLevel {
        SCORE_LEVEL
    }

    fn compute_score(&self, solution: &WorkingSolution) -> Score {
        let problem = solution.problem();

        let mut cost = 0.0;
        for route in solution.routes() {
            cost += route.transport_costs(problem);
        }

        Score::of(self.score_level(), cost * TRANSPORT_COST_WEIGHT)
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();

        let route = context.route();

        let position = match context.insertion {
            Insertion::Service(service_insertion) => service_insertion.position,
            Insertion::Shipment(_) => unimplemented!(),
        };

        let previous_location_id = route.previous_location_id(problem, position);
        let next_location_id = route
            .location_id(problem, position)
            .or_else(|| route.end_location(problem));

        let location_id = match &context.insertion {
            Insertion::Service(service_insertion) => {
                problem.service(service_insertion.job_index).location_id()
            }
            Insertion::Shipment(_) => unimplemented!(),
        };

        let old_cost = problem.travel_cost_or_zero(
            route.vehicle(problem),
            previous_location_id,
            next_location_id,
        );

        let mut new_cost = 0.0;

        new_cost += problem.travel_cost_or_zero(
            route.vehicle(problem),
            previous_location_id,
            Some(location_id),
        );
        new_cost += problem.travel_cost_or_zero(
            route.vehicle(problem),
            Some(location_id),
            next_location_id,
        );

        let travel_cost_delta = new_cost - old_cost;

        Score::of(
            self.score_level(),
            travel_cost_delta * TRANSPORT_COST_WEIGHT,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        problem::job::JobIdx,
        solver::{
            constraints::{
                global_constraint::GlobalConstraint,
                transport_cost_constraint::TransportCostConstraint,
            },
            insertion::{Insertion, ServiceInsertion},
            insertion_context::InsertionContext,
            solution::route_id::RouteIdx,
        },
        test_utils::{self, TestRoute},
    };

    #[test]
    fn test_transport_cost_constraint_insertion_score() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let insertion = Insertion::Service(ServiceInsertion {
            route_id: RouteIdx::new(0),
            job_index: JobIdx::new(6),
            position: 4,
        });

        let constraint = TransportCostConstraint;

        let insertion_context = InsertionContext::new(&problem, &solution, &insertion, false);

        let distances = solution.route(0.into()).transport_costs(&problem);
        let score = constraint.compute_insertion_score(&insertion_context);

        solution.insert(&insertion);

        assert_eq!(
            solution.route(0.into()).transport_costs(&problem),
            distances + score.soft_score,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 6, 4, 5],
        );
    }

    #[test]
    fn test_transport_cost_constraint_insertion_score_end_of_route() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let insertion = Insertion::Service(ServiceInsertion {
            route_id: RouteIdx::new(0),
            job_index: JobIdx::new(6),
            position: 6,
        });

        let constraint = TransportCostConstraint;

        let insertion_context = InsertionContext::new(&problem, &solution, &insertion, false);

        let distances = solution.route(0.into()).transport_costs(&problem);
        let score = constraint.compute_insertion_score(&insertion_context);

        solution.insert(&insertion);

        assert_eq!(
            solution.route(0.into()).transport_costs(&problem),
            distances + score.soft_score,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3, 4, 5, 6],
        );
    }

    #[test]
    fn test_transport_cost_constraint_insertion_score_start_of_route() {
        let locations = test_utils::create_location_grid(10, 10);

        let services = test_utils::create_basic_services(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let mut vehicles = test_utils::create_basic_vehicles(vec![0, 0]);
        vehicles[0].set_should_return_to_depot(true);
        vehicles[1].set_should_return_to_depot(true);
        let problem = Arc::new(test_utils::create_test_problem(
            locations, services, vehicles,
        ));

        let mut solution = test_utils::create_test_working_solution(
            Arc::clone(&problem),
            vec![TestRoute {
                vehicle_id: 0,
                service_ids: vec![0, 1, 2, 3, 4, 5],
            }],
        );

        let insertion = Insertion::Service(ServiceInsertion {
            route_id: RouteIdx::new(0),
            job_index: JobIdx::new(6),
            position: 0,
        });

        let constraint = TransportCostConstraint;

        let insertion_context = InsertionContext::new(&problem, &solution, &insertion, false);

        let distances = solution.route(0.into()).transport_costs(&problem);
        let score = constraint.compute_insertion_score(&insertion_context);

        solution.insert(&insertion);

        assert_eq!(
            solution.route(0.into()).transport_costs(&problem),
            distances + score.soft_score,
        );

        assert_eq!(
            solution
                .route(0.into())
                .activity_ids()
                .iter()
                .map(|activity| activity.job_id().get())
                .collect::<Vec<_>>(),
            vec![6, 0, 1, 2, 3, 4, 5],
        );
    }
}
