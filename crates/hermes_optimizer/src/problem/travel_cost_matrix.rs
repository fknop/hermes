use std::sync::Arc;

use jiff::SignedDuration;
use serde::Deserialize;

use crate::problem::location::LocationIdx;

use super::location::Location;

pub type Distance = f64;
pub type Time = f64;
pub type Cost = f64;

/// This matrix use a flat structure to store distances, times, and costs between locations.
/// To find the index for a pair of locations, use the formula:
/// `index = from * num_locations + to`, where `num_locations` is the total
#[derive(Deserialize)]
pub struct TravelMatrices {
    distances: Arc<Vec<Distance>>,
    times: Arc<Vec<Time>>,
    costs: Arc<Vec<Cost>>,
    num_locations: usize,
    is_symmetric: bool,
}

fn is_flat_matrix_symmetric(matrix: &[f64], num_locations: usize) -> bool {
    for i in 0..num_locations {
        for j in 0..num_locations {
            if matrix[i * num_locations + j] != matrix[j * num_locations + i] {
                return false;
            }
        }
    }
    true
}

impl TravelMatrices {
    pub fn new(
        distances: Vec<Vec<Distance>>,
        times: Vec<Vec<Time>>,
        costs: Vec<Vec<Cost>>,
    ) -> Self {
        let num_locations = distances.len();

        let is_symmetric = distances.iter().enumerate().all(|(i, row)| {
            row.iter()
                .enumerate()
                .all(|(j, &value)| distances[j][i] == value)
        });

        TravelMatrices {
            distances: Arc::new(distances.into_iter().flatten().collect()),
            times: Arc::new(times.into_iter().flatten().collect()),
            costs: Arc::new(costs.into_iter().flatten().collect()),
            num_locations,
            is_symmetric,
        }
    }

    // TODO: later pass the objective
    pub fn from_travel_matrices(
        matrices: hermes_matrix_providers::travel_matrices::TravelMatrices,
    ) -> Self {
        let distances = Arc::new(matrices.distances);
        let times = Arc::new(matrices.times);
        let costs = if let Some(costs) = matrices.costs {
            Arc::new(costs)
        } else {
            Arc::clone(&times)
        };

        let len = distances.len();
        let num_locations = len.isqrt();
        let is_symmetric = is_flat_matrix_symmetric(&distances, num_locations);

        Self {
            distances,
            times,
            costs,
            num_locations,
            is_symmetric,
        }
    }

    #[inline(always)]
    fn index(&self, from: LocationIdx, to: LocationIdx) -> usize {
        from.get() * self.num_locations + to.get()
    }

    pub fn from_haversine(locations: &[Location]) -> Self {
        let num_locations = locations.len();
        let mut distances: Vec<Distance> = vec![0.0; num_locations * num_locations];
        let mut times: Vec<Time> = vec![0.0; num_locations * num_locations];
        // let mut costs: Vec<Cost> = vec![0.0; num_locations * num_locations];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i * num_locations + j] = from.haversine_distance(to);
                // Assume average speed of 50km/h
                let speed = 50.0 / 3.6;
                times[i * num_locations + j] = (distances[i * num_locations + j]) / speed;
                // costs[i * num_locations + j] = distances[i * num_locations + j]
            }
        }

        let distances = Arc::new(distances);
        let costs = Arc::clone(&distances);
        let times = Arc::new(times);

        TravelMatrices {
            distances,
            times,
            costs,
            num_locations,
            is_symmetric: true,
        }
    }

    pub fn from_euclidean(locations: &[Location], round: bool) -> Self {
        let num_locations = locations.len();
        let mut distances: Vec<Distance> = vec![0.0; num_locations * num_locations];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i * num_locations + j] = if round {
                    from.euclidean_distance(to).round()
                } else {
                    from.euclidean_distance(to)
                }
            }
        }

        let distances = Arc::new(distances);
        let costs = Arc::clone(&distances);
        let times = Arc::clone(&distances);

        TravelMatrices {
            distances,
            times,
            costs,
            num_locations,
            is_symmetric: true,
        }
    }

    #[cfg(test)]
    pub fn from_constant(locations: &[Location], time: f64, distance: f64, cost: f64) -> Self {
        let num_locations = locations.len();
        let distances = Arc::new(vec![distance; num_locations * num_locations]);
        let times = Arc::new(vec![time; num_locations * num_locations]);
        let costs = Arc::new(vec![cost; num_locations * num_locations]);
        TravelMatrices {
            distances,
            times,
            costs,
            num_locations,
            is_symmetric: true,
        }
    }

    #[inline(always)]
    pub fn travel_distance(&self, from: LocationIdx, to: LocationIdx) -> Distance {
        if from == to {
            return 0.0;
        }

        self.distances[self.index(from, to)]
    }

    #[inline(always)]
    pub fn travel_time(&self, from: LocationIdx, to: LocationIdx) -> SignedDuration {
        if from == to {
            return SignedDuration::ZERO;
        }

        SignedDuration::from_secs_f64(self.times[self.index(from, to)])
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: LocationIdx, to: LocationIdx) -> Cost {
        if from == to {
            return 0.0;
        }

        self.costs[self.index(from, to)]
    }

    pub fn max_cost(&self) -> Cost {
        self.costs.iter().cloned().fold(0.0, f64::max)
    }

    pub fn is_symmetric(&self) -> bool {
        self.is_symmetric
    }

    pub fn num_locations(&self) -> usize {
        self.num_locations
    }

    pub(super) fn times(&self) -> &[Time] {
        &self.times
    }

    pub(super) fn distances(&self) -> &[Distance] {
        &self.distances
    }

    pub(super) fn costs(&self) -> &[Cost] {
        &self.costs
    }
}
