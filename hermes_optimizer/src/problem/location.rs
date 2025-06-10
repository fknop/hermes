pub type LocationId = usize;
pub struct Location {
    id: LocationId,
    x: f64,
    y: f64,
}

impl Location {
    pub fn id(&self) -> LocationId {
        self.id
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }
}
