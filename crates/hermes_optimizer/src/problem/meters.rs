use std::{
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub, SubAssign},
};

use jiff::SignedDuration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::problem::kmh::Kmh;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, JsonSchema)]
pub struct Meters(f64);

impl Meters {
    pub const ZERO: Meters = Meters(0.0);

    pub fn new(value: f64) -> Self {
        Meters(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }
}

impl Eq for Meters {}

impl PartialOrd for Meters {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Meters {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl From<f64> for Meters {
    fn from(value: f64) -> Self {
        Meters::new(value)
    }
}

impl Add for Meters {
    type Output = Meters;

    fn add(self, other: Meters) -> Meters {
        Meters(self.0 + other.0)
    }
}

impl AddAssign for Meters {
    fn add_assign(&mut self, other: Meters) {
        self.0 += other.0;
    }
}

impl Sub for Meters {
    type Output = Meters;

    fn sub(self, other: Meters) -> Meters {
        Meters(self.0 - other.0)
    }
}

impl SubAssign for Meters {
    fn sub_assign(&mut self, other: Meters) {
        self.0 -= other.0;
    }
}

impl Div<Kmh> for Meters {
    type Output = SignedDuration;

    fn div(self, speed: Kmh) -> SignedDuration {
        let seconds = self.0 * 3.6 / speed.value();
        SignedDuration::from_secs_f64(seconds)
    }
}

impl Div<usize> for Meters {
    type Output = Meters;

    fn div(self, rhs: usize) -> Meters {
        Meters(self.0 / rhs as f64)
    }
}

impl Div<Meters> for Meters {
    type Output = f64;

    fn div(self, other: Meters) -> f64 {
        self.0 / other.0
    }
}

impl Sum for Meters {
    fn sum<I: Iterator<Item = Meters>>(iter: I) -> Meters {
        iter.fold(Meters::ZERO, |acc, x| acc + x)
    }
}
