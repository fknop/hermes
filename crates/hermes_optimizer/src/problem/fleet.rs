use crate::problem::vehicle::{Vehicle, VehicleId};

pub enum Fleet {
    Finite(Vec<Vehicle>),
    Infinite(Vec<Vehicle>),
}

impl Fleet {
    pub fn is_infinite(&self) -> bool {
        matches!(self, Fleet::Infinite(_))
    }

    #[inline]
    pub fn vehicles(&self) -> &[Vehicle] {
        match self {
            Fleet::Finite(vehicles) => vehicles,
            Fleet::Infinite(vehicles) => vehicles,
        }
    }

    #[inline]
    pub fn vehicle(&self, vehicle_id: VehicleId) -> &Vehicle {
        match self {
            Fleet::Finite(vehicles) => &vehicles[vehicle_id],
            Fleet::Infinite(vehicles) => &vehicles[vehicle_id],
        }
    }
}
