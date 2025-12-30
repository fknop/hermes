use serde::{Deserialize, Serialize};

use crate::{graphhopper_api::GraphHopperProfile, travel_matrices::TravelMatrices};

#[derive(Deserialize, Serialize)]
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

impl std::hash::Hash for TravelMatrixProvider {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TravelMatrixProvider::GraphHopperApi { gh_profile } => {
                state.write_u8(0);
                gh_profile.hash(state);
            }
            TravelMatrixProvider::AsTheCrowFlies { speed_kmh } => {
                state.write_u8(1);
                state.write_u64(speed_kmh.to_bits());
            }
            TravelMatrixProvider::Custom { matrices } => {
                state.write_u8(2);
                matrices.hash(state);
            }
        }
    }
}
