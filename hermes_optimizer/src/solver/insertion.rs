use crate::problem::{service::ServiceId, vehicle::VehicleId};

pub struct ExistingRouteInsertion {
    pub route_id: usize,
    pub service_id: ServiceId,
    pub position: usize,
}

pub struct NewRouteInsertion {
    pub service_id: ServiceId,
    pub vehicle_id: VehicleId,
}

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
}
