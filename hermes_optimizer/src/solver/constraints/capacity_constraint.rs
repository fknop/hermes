use std::ops::AddAssign;

use crate::{
    problem::{
        capacity::Capacity, service::ServiceType, vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{ExistingRouteInsertion, Insertion, NewRouteInsertion},
        insertion_context::InsertionContext,
        score::Score,
        score_level::ScoreLevel,
        solution::route::WorkingSolutionRoute,
    },
};

use super::route_constraint::RouteConstraint;

pub struct CapacityConstraint {
    score_level: ScoreLevel,
}

impl Default for CapacityConstraint {
    fn default() -> Self {
        CapacityConstraint {
            score_level: ScoreLevel::Hard,
        }
    }
}

impl CapacityConstraint {
    pub fn new(score_level: ScoreLevel) -> Self {
        CapacityConstraint { score_level }
    }
}

impl CapacityConstraint {
    fn compute_capacity_score(
        &self,
        load: &mut Capacity,
        vehicle_capacity: &Capacity,
        initial_load: &Capacity,
        cumulative_load: &Capacity,
    ) -> Score {
        load.reset();
        load.add_assign(initial_load);
        load.add_assign(cumulative_load);
        if vehicle_capacity.satisfies_demand(load) {
            Score::zero()
        } else {
            Score::of(
                self.score_level,
                vehicle_capacity.over_capacity_demand(load),
            )
        }
    }
}

impl RouteConstraint for CapacityConstraint {
    fn score_level(&self) -> ScoreLevel {
        self.score_level
    }

    fn compute_score(
        &self,
        problem: &VehicleRoutingProblem,
        route: &WorkingSolutionRoute,
    ) -> Score {
        let vehicle = route.vehicle(problem);
        let initial_load = route.total_initial_load();

        let mut score = Score::zero();
        if !vehicle.capacity().satisfies_demand(initial_load) {
            score += Score::of(
                self.score_level,
                vehicle.capacity().over_capacity_demand(initial_load),
            );
        }

        // Reuse vector to avoid allocations
        let mut load = Capacity::zero();
        for activity in route.activities() {
            score += self.compute_capacity_score(
                &mut load,
                vehicle.capacity(),
                initial_load,
                activity.cumulative_load(),
            );
        }

        score
    }

    fn compute_insertion_score(&self, context: &InsertionContext) -> Score {
        let problem = context.problem();
        let service = problem.service(context.insertion.service_id());

        if service.demand().is_empty() {
            return Score::zero();
        }

        let vehicle = match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion { route_id, .. }) => {
                context.solution.route(route_id).vehicle(problem)
            }
            Insertion::NewRoute(NewRouteInsertion { vehicle_id, .. }) => {
                problem.vehicle(vehicle_id)
            }
        };

        let mut score = Score::zero();
        if !vehicle.capacity().satisfies_demand(&context.initial_load) {
            score += Score::of(
                self.score_level,
                vehicle
                    .capacity()
                    .over_capacity_demand(&context.initial_load),
            );
        }

        match *context.insertion {
            Insertion::ExistingRoute(ExistingRouteInsertion {
                route_id,
                position,
                service_id,
            }) => {
                let service = problem.service(service_id);
                if let Some(next_activity) =
                    context.solution.route(route_id).activities().get(position)
                    && service.service_type() == ServiceType::Pickup
                {
                    let new_max_load = next_activity.max_load_until_end() + service.demand();
                    if !vehicle.capacity().satisfies_demand(&new_max_load) {
                        score += Score::of(
                            self.score_level,
                            vehicle.capacity().over_capacity_demand(&new_max_load),
                        );
                    }
                }
            }
            Insertion::NewRoute(NewRouteInsertion { .. }) => {
                // No activities before insertion in new route
            }
        }

        score
    }
}
