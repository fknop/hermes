use crate::{
    define_index_newtype, problem::vehicle::VehicleId,
    solver::solution::route::WorkingSolutionRoute,
};

define_index_newtype!(RouteId, WorkingSolutionRoute);

// Temporary conversion from VehicleId to RouteId
//
impl From<VehicleId> for RouteId {
    fn from(vehicle_id: VehicleId) -> Self {
        RouteId(vehicle_id.get())
    }
}
