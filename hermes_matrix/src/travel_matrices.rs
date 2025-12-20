use serde::Deserialize;

/// TravelMatrices holds the travel distance, time, and cost matrices.
/// Stored as flat vectors
/// Stored as Rc to allow sharing the vectors without cloning.
#[derive(Deserialize)]
pub struct TravelMatrices {
    pub distances: Vec<f64>,
    pub times: Vec<f64>,

    // Some providers don't give use a cost
    pub costs: Option<Vec<f64>>,
}
