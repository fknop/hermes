use jiff::SignedDuration;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::problem::{capacity::Capacity, location::LocationId, time_window::TimeWindow};

type TimeWindows = SmallVec<[TimeWindow; 1]>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ShipmentLocation {
    duration: SignedDuration,
    location_id: LocationId,
    time_windows: TimeWindows,
}

impl ShipmentLocation {
    pub fn duration(&self) -> SignedDuration {
        self.duration
    }

    pub fn location_id(&self) -> LocationId {
        self.location_id
    }

    pub fn time_windows_satisfied(&self, arrival_time: jiff::Timestamp) -> bool {
        self.time_windows
            .iter()
            .any(|tw| tw.is_satisfied(arrival_time))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Shipment {
    external_id: String,
    demand: Capacity,
    pickup: ShipmentLocation,
    delivery: ShipmentLocation,
}

impl Shipment {
    pub fn demand(&self) -> &Capacity {
        &self.demand
    }

    pub fn pickup(&self) -> &ShipmentLocation {
        &self.pickup
    }

    pub fn delivery(&self) -> &ShipmentLocation {
        &self.delivery
    }
}
