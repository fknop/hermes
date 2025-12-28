use fxhash::FxHashSet;
use jiff::SignedDuration;
use serde::Serialize;
use smallvec::SmallVec;

use crate::problem::{
    capacity::Capacity, location::LocationIdx, skill::Skill, time_window::TimeWindow,
};

type TimeWindows = SmallVec<[TimeWindow; 1]>;

#[derive(Serialize, Debug, Clone)]
pub struct ShipmentLocation {
    duration: SignedDuration,
    location_id: LocationIdx,
    time_windows: TimeWindows,
}

impl ShipmentLocation {
    pub fn duration(&self) -> SignedDuration {
        self.duration
    }

    pub fn location_id(&self) -> LocationIdx {
        self.location_id
    }

    pub fn time_windows(&self) -> &TimeWindows {
        &self.time_windows
    }

    pub fn time_windows_satisfied(&self, arrival_time: jiff::Timestamp) -> bool {
        self.time_windows
            .iter()
            .any(|tw| tw.is_satisfied(arrival_time))
    }

    pub fn has_time_windows(&self) -> bool {
        self.time_windows.iter().any(|tw| !tw.is_empty())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Shipment {
    external_id: String,
    demand: Capacity,
    pickup: ShipmentLocation,
    delivery: ShipmentLocation,
    skills: FxHashSet<Skill>,
}

impl Shipment {
    pub fn skills(&self) -> &FxHashSet<Skill> {
        &self.skills
    }

    pub fn external_id(&self) -> &str {
        &self.external_id
    }

    pub fn demand(&self) -> &Capacity {
        &self.demand
    }

    pub fn pickup(&self) -> &ShipmentLocation {
        &self.pickup
    }

    pub fn delivery(&self) -> &ShipmentLocation {
        &self.delivery
    }

    pub fn has_time_windows(&self) -> bool {
        self.pickup.time_windows.iter().any(|tw| !tw.is_empty())
            || self.delivery.time_windows.iter().any(|tw| !tw.is_empty())
    }
}
