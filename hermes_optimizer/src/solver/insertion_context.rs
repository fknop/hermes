use crate::problem::{service::ServiceId, vehicle::VehicleId};

pub struct ExistingRouteInsertionContext {
    pub route_id: usize,
    pub service_id: ServiceId,
    pub position: usize,
}

pub struct NewRouteInsertionContext {
    pub service_id: ServiceId,
    pub vehicle_id: VehicleId,
}

pub enum InsertionContext {
    NewRoute(NewRouteInsertionContext),
    ExistingRoute(ExistingRouteInsertionContext),
}

impl InsertionContext {
    pub fn service_id(&self) -> ServiceId {
        match self {
            InsertionContext::NewRoute(ctx) => ctx.service_id,
            InsertionContext::ExistingRoute(ctx) => ctx.service_id,
        }
    }

    pub fn position(&self) -> usize {
        match self {
            InsertionContext::NewRoute(_) => 0,
            InsertionContext::ExistingRoute(ctx) => ctx.position,
        }
    }
}
