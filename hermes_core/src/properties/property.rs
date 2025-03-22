#[derive(Eq, Hash, PartialEq, Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub enum Property {
    MaxSpeed,
    VehicleAccess(String),
    OsmId,
}

impl Property {
    pub fn as_string(&self) -> String {
        match self {
            Property::MaxSpeed => "maxspeed".to_string(),
            Property::VehicleAccess(access) => format!("vehicle_access_{}", access),
            Property::OsmId => "osm_id".to_string(),
        }
    }
}
