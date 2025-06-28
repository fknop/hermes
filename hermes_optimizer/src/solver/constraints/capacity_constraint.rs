use crate::solver::{
    insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
    insertion_context::{ActivityInsertionContext, InsertionContext},
    score::Score,
    working_solution::{WorkingSolution, WorkingSolutionRoute, WorkingSolutionRouteActivity},
};

use super::route_constraint::RouteConstraint;

pub struct CapacityConstraint;

impl RouteConstraint for CapacityConstraint {
    fn compute_score(&self, route: &WorkingSolutionRoute) -> Score {
        let vehicle = route.vehicle();
        let total_demand = route.total_demand();

        if vehicle.capacity().satisfies_demand(total_demand) {
            Score::zero()
        } else {
            Score::hard(
                vehicle
                    .capacity()
                    .over_capacity_demand(total_demand)
                    .round() as i64,
            )
        }
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.solution.problem();
        let service = problem.service(context.insertion.service_id());

        if service.demand().is_empty() {
            return Score::zero();
        }

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                let route = context.solution.route(route_id);
                let vehicle = route.vehicle();
                let current_demand = route.total_demand();

                let new_demand = current_demand + service.demand();
                if vehicle.capacity().satisfies_demand(&new_demand) {
                    Score::zero()
                } else {
                    Score::hard(vehicle.capacity().over_capacity_demand(&new_demand).round() as i64)
                }
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                let vehicle = problem.vehicle(vehicle_id);
                if vehicle.capacity().satisfies_demand(service.demand()) {
                    Score::zero()
                } else {
                    Score::hard(
                        vehicle
                            .capacity()
                            .over_capacity_demand(service.demand())
                            .round() as i64,
                    )
                }
            }
        }
    }
}
