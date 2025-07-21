use crate::{
    problem::vehicle_routing_problem::VehicleRoutingProblem,
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        working_solution::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct CapacityConstraint;

impl RouteConstraint for CapacityConstraint {
    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        let total_demand = route.total_demand();

        if vehicle.capacity().satisfies_demand(total_demand) {
            Score::zero()
        } else {
            Score::hard(vehicle.capacity().over_capacity_demand(total_demand))
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let service = problem.service(context.insertion.service_id());

        if service.demand().is_empty() {
            return Score::zero();
        }

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle(problem);
                let current_demand = route.total_demand();

                let new_demand = current_demand + service.demand();
                if vehicle.capacity().satisfies_demand(&new_demand) {
                    Score::zero()
                } else {
                    Score::hard(vehicle.capacity().over_capacity_demand(&new_demand))
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);
                if vehicle.capacity().satisfies_demand(service.demand()) {
                    Score::zero()
                } else {
                    Score::hard(vehicle.capacity().over_capacity_demand(service.demand()))
                }
            }
        }
    }
}
