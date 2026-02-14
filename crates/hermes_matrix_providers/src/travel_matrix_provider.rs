use hermes_graphhopper::client::GraphHopperProfile;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CustomMatrices {
    pub times: Vec<Vec<f64>>,
    pub distances: Vec<Vec<f64>>,
    pub costs: Vec<Vec<f64>>,
}

impl std::hash::Hash for CustomMatrices {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for d in self.distances.iter().flatten() {
            state.write_u64(d.to_bits());
        }
        for t in self.times.iter().flatten() {
            state.write_u64(t.to_bits());
        }

        for c in self.costs.iter().flatten() {
            state.write_u64(c.to_bits());
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", content = "config", rename_all = "snake_case")]
pub enum TravelMatrixProvider {
    /// https://docs.graphhopper.com/openapi/map-data-and-routing-profiles/openstreetmap/standard-routing-profiles
    GraphHopperApi {
        gh_profile: GraphHopperProfile,
    },
    Osrm {
        profile: String,
    },
    // Valhalla { profile: String },
    AsTheCrowFlies {
        speed_kmh: f64,
    },

    Custom {
        matrices: CustomMatrices,
    },
}

impl std::hash::Hash for TravelMatrixProvider {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TravelMatrixProvider::GraphHopperApi { gh_profile } => {
                state.write_u8(0);
                gh_profile.hash(state);
            }
            TravelMatrixProvider::Osrm { profile } => {
                state.write_u8(1);
                profile.hash(state);
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
