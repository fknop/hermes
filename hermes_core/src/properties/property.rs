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
    CarAverageSpeed,
    CarVehicleAccess,
    OsmId,
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Property::MaxSpeed => write!(f, "maxspeed"),
            Property::CarAverageSpeed => write!(f, "car_average_speed"),
            Property::CarVehicleAccess => write!(f, "car_vehicle_access"),
            Property::OsmId => write!(f, "osm_id"),
        }
    }
}
