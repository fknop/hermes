use crate::{
    problem::{service::ServiceId, vehicle::VehicleId},
    solver::solution::{route::WorkingSolutionRoute, working_solution::WorkingSolution},
};

#[derive(Clone)]
pub struct ExistingRouteInsertion {
    pub route_id: usize,
    pub service_id: ServiceId,
    pub position: usize,
}

#[derive(Clone)]
pub struct NewRouteInsertion {
    pub service_id: ServiceId,
    pub vehicle_id: VehicleId,
}

#[derive(Clone)]
pub enum Insertion {
    NewRoute(NewRouteInsertion),
    ExistingRoute(ExistingRouteInsertion),
}

impl Insertion {
    pub fn service_id(&self) -> ServiceId {
        match self {
            Insertion::NewRoute(ctx) => ctx.service_id,
            Insertion::ExistingRoute(ctx) => ctx.service_id,
        }
    }

    pub fn position(&self) -> usize {
        match self {
            Insertion::NewRoute(_) => 0,
            Insertion::ExistingRoute(ctx) => ctx.position,
        }
    }

    pub fn route<'a>(&self, solution: &'a WorkingSolution) -> &'a WorkingSolutionRoute {
        match self {
            Insertion::NewRoute(ctx) => solution.route(ctx.vehicle_id),
            Insertion::ExistingRoute(ctx) => solution.route(ctx.route_id),
        }
    }
}
