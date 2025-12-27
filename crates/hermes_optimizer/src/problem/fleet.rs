use crate::problem::vehicle::Vehicle;

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
    pub fn vehicle(&self, index: usize) -> &Vehicle {
        match self {
            Fleet::Finite(vehicles) => &vehicles[index],
            Fleet::Infinite(vehicles) => &vehicles[index],
        }
    }
}
