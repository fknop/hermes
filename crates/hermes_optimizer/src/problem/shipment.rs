use fxhash::FxHashSet;
use jiff::SignedDuration;
use serde::Serialize;
use smallvec::SmallVec;

use crate::{
    problem::{
        capacity::Capacity,
        location::LocationIdx,
        skill::Skill,
        time_window::{TimeWindow, TimeWindows},
    },
    utils::bitset::BitSet,
};

#[derive(Serialize, Debug, Clone)]
pub struct ShipmentLocation {
    duration: SignedDuration,
    location_id: LocationIdx,
    time_windows: TimeWindows,
}

impl ShipmentLocation {
    pub fn new(
        duration: SignedDuration,
        location_id: LocationIdx,
        time_windows: TimeWindows,
    ) -> Self {
        Self {
            duration,
            location_id,
            time_windows,
        }
    }

    pub fn duration(&self) -> SignedDuration {
        self.duration
    }

    pub fn location_id(&self) -> LocationIdx {
        self.location_id
    }

    pub fn time_windows(&self) -> &TimeWindows {
        &self.time_windows
    }

    pub fn has_time_windows(&self) -> bool {
        !self.time_windows.is_empty()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Shipment {
    external_id: String,
    demand: Capacity,
    pickup: ShipmentLocation,
    delivery: ShipmentLocation,
    skills: FxHashSet<Skill>,
    #[serde(skip)]
    skills_bitset: BitSet,
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
        !self.pickup.time_windows.is_empty() || !self.delivery.time_windows.is_empty()
    }

    pub fn skills_bitset(&self) -> &BitSet {
        &self.skills_bitset
    }

    pub fn set_skills_bitset(&mut self, skills_bitset: BitSet) {
        self.skills_bitset = skills_bitset;
    }
}

#[derive(Default)]
pub struct ShipmentBuilder {
    external_id: Option<String>,
    demand: Option<Capacity>,
    pickup_location_id: Option<usize>,
    pickup_duration: Option<SignedDuration>,
    pickup_time_windows: Option<Vec<TimeWindow>>,
    delivery_location_id: Option<usize>,
    delivery_duration: Option<SignedDuration>,
    delivery_time_windows: Option<Vec<TimeWindow>>,
}

impl ShipmentBuilder {
    pub fn set_external_id(&mut self, id: String) -> &mut Self {
        self.external_id = Some(id);
        self
    }

    pub fn set_demand(&mut self, demand: Capacity) -> &mut Self {
        self.demand = Some(demand);
        self
    }

    pub fn set_pickup_location_id(&mut self, id: usize) -> &mut Self {
        self.pickup_location_id = Some(id);
        self
    }

    pub fn set_pickup_duration(&mut self, duration: SignedDuration) -> &mut Self {
        self.pickup_duration = Some(duration);
        self
    }

    pub fn set_pickup_time_window(&mut self, tw: TimeWindow) -> &mut Self {
        if let Some(tws) = &mut self.pickup_time_windows {
            tws.push(tw);
        } else {
            self.pickup_time_windows = Some(vec![tw]);
        }
        self
    }

    pub fn set_delivery_location_id(&mut self, id: usize) -> &mut Self {
        self.delivery_location_id = Some(id);
        self
    }

    pub fn set_delivery_duration(&mut self, duration: SignedDuration) -> &mut Self {
        self.delivery_duration = Some(duration);
        self
    }

    pub fn set_delivery_time_window(&mut self, tw: TimeWindow) -> &mut Self {
        if let Some(tws) = &mut self.delivery_time_windows {
            tws.push(tw);
        } else {
            self.delivery_time_windows = Some(vec![tw]);
        }
        self
    }

    pub fn build(self) -> Shipment {
        let pickup = ShipmentLocation {
            duration: self.pickup_duration.unwrap_or(SignedDuration::ZERO),
            location_id: self
                .pickup_location_id
                .expect("Expected pickup location id")
                .into(),
            time_windows: TimeWindows::new(SmallVec::from_vec(
                self.pickup_time_windows.unwrap_or_default(),
            )),
        };

        let delivery = ShipmentLocation {
            duration: self.delivery_duration.unwrap_or(SignedDuration::ZERO),
            location_id: self
                .delivery_location_id
                .expect("Expected delivery location id")
                .into(),
            time_windows: TimeWindows::new(SmallVec::from_vec(
                self.delivery_time_windows.unwrap_or_default(),
            )),
        };

        Shipment {
            external_id: self.external_id.expect("Expected external id"),
            demand: self.demand.unwrap_or_default(),
            pickup,
            delivery,
            skills: FxHashSet::default(),
            skills_bitset: BitSet::empty(),
        }
    }
}
