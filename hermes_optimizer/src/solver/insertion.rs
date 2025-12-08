use crate::{
    problem::{job::ActivityId, service::ServiceId, vehicle::VehicleId},
    solver::solution::{route::WorkingSolutionRoute, working_solution::WorkingSolution},
};

#[derive(Clone)]
pub struct ServiceInsertion {
    pub route_id: usize,
    pub job_index: usize,
    pub position: usize,
}

#[derive(Clone)]
pub struct ShipmentInsertion {
    pub route_id: usize,
    pub job_index: usize,

    /// Position of the pickup
    pub pickup_position: usize,

    /// This is the position before the pickup has been inserted
    pub delivery_position: usize,
}

#[derive(Clone)]
pub enum Insertion {
    Service(ServiceInsertion),
    Shipment(ShipmentInsertion),
}

impl Insertion {
    pub fn job_index(&self) -> usize {
        match self {
            Insertion::Service(ctx) => ctx.job_index,
            Insertion::Shipment(ctx) => ctx.job_index,
        }
    }

    // pub fn position(&self) -> usize {
    //     match self {
    //         Insertion::NewRoute(_) => 0,
    //         Insertion::ExistingRoute(ctx) => ctx.position,
    //     }
    // }

    pub fn route<'a>(&self, solution: &'a WorkingSolution) -> &'a WorkingSolutionRoute {
        match self {
            Insertion::Service(ctx) => solution.route(ctx.route_id),
            Insertion::Shipment(ctx) => solution.route(ctx.route_id),
        }
    }
}
