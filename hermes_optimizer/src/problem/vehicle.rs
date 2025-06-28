use jiff::Timestamp;

use super::{capacity::Capacity, location::LocationId};

pub type VehicleId = usize;

pub struct Vehicle {
    external_id: String,
    shift: Option<VehicleShift>,
    capacity: Capacity,
    depot_location_id: Option<usize>,
    should_return_to_depot: bool,
}

impl Vehicle {
    pub fn capacity(&self) -> &Capacity {
        &self.capacity
    }

    pub fn depot_location_id(&self) -> Option<usize> {
        self.depot_location_id
    }

    pub fn earliest_start_time(&self) -> Option<Timestamp> {
        self.shift.as_ref().and_then(|shift| shift.earliest_start)
    }

    pub fn latest_end_time(&self) -> Option<Timestamp> {
        self.shift.as_ref().and_then(|shift| shift.latest_end)
    }

    pub fn should_return_to_depot(&self) -> bool {
        self.should_return_to_depot
    }

    pub fn set_shift(&mut self, shift: VehicleShift) {
        self.shift = Some(shift);
    }

    pub fn set_depot_location(&mut self, location_id: LocationId) {
        self.depot_location_id = Some(location_id);
    }
}

pub struct VehicleShift {
    earliest_start: Option<Timestamp>,
    latest_end: Option<Timestamp>,
}

impl VehicleShift {
    pub fn new(earliest_start: Option<Timestamp>, latest_end: Option<Timestamp>) -> Self {
        VehicleShift {
            earliest_start,
            latest_end,
        }
    }

    pub fn earliest_start(&self) -> Option<Timestamp> {
        self.earliest_start
    }

    pub fn latest_end(&self) -> Option<Timestamp> {
        self.latest_end
    }
}

#[derive(Default)]
pub struct VehicleBuilder {
    external_id: Option<String>,
    shift: Option<VehicleShift>,
    capacity: Option<Capacity>,
    depot_location_id: Option<usize>,
    should_return_to_depot: Option<bool>,
}

impl VehicleBuilder {
    pub fn with_vehicle_id(mut self, external_id: String) -> Self {
        self.external_id = Some(external_id);
        self
    }

    pub fn with_vehicle_shift(mut self, shift: VehicleShift) -> Self {
        self.shift = Some(shift);
        self
    }

    pub fn with_capacity(mut self, capacity: Capacity) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn with_depot_location_id(mut self, depot_location_id: usize) -> Self {
        self.depot_location_id = Some(depot_location_id);
        self
    }

    pub fn with_return(mut self, should_return_to_depot: bool) -> Self {
        // This method is not used in the current implementation but can be added for future use
        self.should_return_to_depot = Some(should_return_to_depot);
        self
    }

    pub fn build(self) -> Vehicle {
        Vehicle {
            external_id: self.external_id.expect("External ID is required"),
            shift: self.shift,
            capacity: self.capacity.unwrap_or_else(|| Capacity::ZERO),
            depot_location_id: self.depot_location_id,
            should_return_to_depot: self.should_return_to_depot.unwrap_or(false),
        }
    }
}
