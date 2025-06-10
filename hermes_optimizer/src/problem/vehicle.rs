use std::time::Instant;

pub type Capacity = f64;

pub struct Vehicle {
    id: String,
    shift: VehicleShift,
    capacity: Vec<Capacity>,
}

pub struct VehicleShift {
    earliest_start: Option<Instant>,
    latest_end: Option<Instant>,
}
