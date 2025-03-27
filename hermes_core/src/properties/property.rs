#[derive(Eq, Hash, PartialEq, Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Property {
    MaxSpeed,
    AverageSpeed(String),
    VehicleAccess(String),
    OsmId,
}

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
