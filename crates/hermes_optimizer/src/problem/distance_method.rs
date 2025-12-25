use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DistanceMethod {
    Haversine,
    Euclidean,
}
