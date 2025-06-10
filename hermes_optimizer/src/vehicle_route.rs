use std::time::Instant;

use crate::problem::service::ServiceId;

pub struct VehicleRoute {
    activities: Vec<VehicleRouteActivity>,
}

pub struct VehicleRouteActivity {
    service_id: ServiceId,
    arrival_time: Instant,
}
