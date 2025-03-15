#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Property {
    MaxSpeed,
    VehicleAccess(String),
}
