use jiff::Timestamp;

use crate::{
    problem::{
        job::ActivityId,
        vehicle_routing_problem::VehicleRoutingProblem,
    },
    solver::{
        insertion::{ServiceInsertion, ShipmentInsertion},
        solution::{
            route::WorkingSolutionRoute,
            route_update_iterator::RouteUpdateIterator,
            utils::{
                compute_first_activity_arrival_time, compute_vehicle_end, compute_vehicle_start,
            },
        },
    },
};

use super::{insertion::Insertion, solution::working_solution::WorkingSolution};

pub struct InsertionContext<'a> {
    pub problem: &'a VehicleRoutingProblem,
    pub solution: &'a WorkingSolution,
    pub insertion: &'a Insertion,
}

impl<'a> InsertionContext<'a> {
    pub fn new(
        problem: &'a VehicleRoutingProblem,
        solution: &'a WorkingSolution,
        insertion: &'a Insertion,
    ) -> Self {
        InsertionContext {
            problem,
            solution,
            insertion,
        }
    }

    pub fn problem(&self) -> &VehicleRoutingProblem {
        self.problem
    }

    pub fn route(&self) -> &WorkingSolutionRoute {
        self.insertion.route(self.solution)
    }

    pub fn compute_vehicle_start(&self) -> Timestamp {
        let route = self.insertion.route(self.solution);

        match *self.insertion {
            Insertion::Service(ServiceInsertion {
                route_id: _,
                job_index,
                position,
            }) => {
                if position == 0 {
                    let job_id = ActivityId::Service(job_index);
                    compute_vehicle_start(
                        self.problem,
                        route.vehicle_id(),
                        job_id,
                        compute_first_activity_arrival_time(
                            self.problem,
                            route.vehicle_id(),
                            job_id,
                        ),
                    )
                } else {
                    route.start(self.problem)
                }
            }
            Insertion::Shipment(ShipmentInsertion {
                job_index,
                pickup_position,
                route_id: _,
                ..
            }) => {
                if pickup_position == 0 {
                    let job_id = ActivityId::ShipmentPickup(job_index);
                    compute_vehicle_start(
                        self.problem,
                        route.vehicle_id(),
                        job_id,
                        compute_first_activity_arrival_time(
                            self.problem,
                            route.vehicle_id(),
                            job_id,
                        ),
                    )
                } else {
                    route.start(self.problem)
                }
            }
        }
    }

    pub fn updated_activities_iter(
        &'a self,
    ) -> RouteUpdateIterator<'a, Box<dyn Iterator<Item = ActivityId> + 'a>> {
        let route = self.insertion.route(self.solution);

        match *self.insertion {
            Insertion::Service(ServiceInsertion {
                job_index,
                position,
                ..
            }) => {
                let activity_id = ActivityId::Service(job_index);

                route.updated_activities_iter(
                    self.problem,
                    Box::new(
                        std::iter::once(activity_id)
                            .chain(route.job_ids_iter(position, route.len())),
                    ),
                    position,
                    route.len() + 1,
                )
            }
            Insertion::Shipment(ShipmentInsertion {
                job_index,
                pickup_position,
                delivery_position,
                ..
            }) => route.updated_activities_iter(
                self.problem,
                Box::new(
                    std::iter::once(ActivityId::ShipmentPickup(job_index))
                        .chain(route.job_ids_iter(pickup_position, delivery_position))
                        .chain(std::iter::once(ActivityId::ShipmentDelivery(job_index)))
                        .chain(route.job_ids_iter(delivery_position, route.len())),
                ),
                pickup_position,
                route.len() + 2,
            ),
        }
    }

    pub fn compute_vehicle_end(&self) -> Timestamp {
        let route = self.insertion.route(self.solution);

        let last_service = self.updated_activities_iter().last().unwrap();

        compute_vehicle_end(
            self.problem,
            route.vehicle_id(),
            last_service.job_id,
            last_service.departure_time,
        )
    }
}
