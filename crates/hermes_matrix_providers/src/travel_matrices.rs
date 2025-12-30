use serde::{Deserialize, Serialize};

/// TravelMatrices holds the travel distance, time, and cost matrices.
/// Stored as flat vectors
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TravelMatrices {
    pub distances: Vec<f64>,
    pub times: Vec<f64>,

    // Some providers don't give use a cost
    pub costs: Option<Vec<f64>>,
}

impl std::hash::Hash for TravelMatrices {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for d in &self.distances {
            state.write_u64(d.to_bits());
        }
        for t in &self.times {
            state.write_u64(t.to_bits());
        }
        if let Some(costs) = &self.costs {
            for c in costs {
                state.write_u64(c.to_bits());
            }
        } else {
            state.write_u8(0);
        }
    }
}
