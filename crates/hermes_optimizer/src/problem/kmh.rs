use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize)]
pub struct Kmh(f64);

impl Kmh {
    pub fn new(value: f64) -> Self {
        Kmh(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}
