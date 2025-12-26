use serde::Deserialize;

use crate::{graphhopper_api::GraphHopperProfile, travel_matrices::TravelMatrices};

#[derive(Deserialize)]
pub enum TravelMatrixProvider {
    /// https://docs.graphhopper.com/openapi/map-data-and-routing-profiles/openstreetmap/standard-routing-profiles
    GraphHopperApi {
        gh_profile: GraphHopperProfile,
    },
    // OSRM { profile: String },
    // Valhalla { profile: String },
    AsTheCrowFlies {
        speed_kmh: f64,
    },

    Custom {
        matrices: TravelMatrices,
    },
}
