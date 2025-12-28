use crate::{
    define_index_newtype, problem::vehicle::VehicleIdx,
    solver::solution::route::WorkingSolutionRoute,
};

define_index_newtype!(RouteIdx, WorkingSolutionRoute);

// Temporary conversion from VehicleId to RouteId

impl From<VehicleIdx> for RouteIdx {
    fn from(vehicle_id: VehicleIdx) -> Self {
        RouteIdx(vehicle_id.get())
    }
}
