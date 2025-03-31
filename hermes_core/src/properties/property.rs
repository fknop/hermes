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

// TODO: fix ToString -> Display
impl ToString for Property {
    fn to_string(&self) -> String {
        match self {
            Property::MaxSpeed => "maxspeed".to_string(),
            Property::AverageSpeed(vehicle_type) => format!("{}_average_speed", vehicle_type),
            Property::VehicleAccess(vehicle_type) => format!("{}_vehicle_access", vehicle_type),
            Property::OsmId => "osm_id".to_string(),
        }
    }
}
