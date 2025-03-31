#[derive(Eq, Hash, PartialEq, Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(
    // This will generate a PartialEq impl between our unarchived
    // and archived types
    compare(PartialEq),
    // Derives can be passed through to the generated type:
    derive(Debug),
)]
pub enum Property {
    MaxSpeed,
    AverageSpeed(String),
    VehicleAccess(String),
    OsmId,
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Property::MaxSpeed => write!(f, "maxspeed"),
            Property::AverageSpeed(vehicle_type) => write!(f, "{}_average_speed", vehicle_type),
            Property::VehicleAccess(vehicle_type) => write!(f, "{}_vehicle_access", vehicle_type),
            Property::OsmId => write!(f, "osm_id"),
        }
    }
}
