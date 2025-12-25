use std::{rc::Rc, sync::Arc};

use jiff::SignedDuration;
use serde::Deserialize;

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

    #[inline(always)]
    fn get_index(&self, from: usize, to: usize) -> usize {
        from * self.num_locations + to
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

    pub fn from_euclidian(locations: &[Location]) -> Self {
        let num_locations = locations.len();
        let mut distances: Vec<Distance> = vec![0.0; num_locations * num_locations];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i * num_locations + j] = from.euclidian_distance(to);
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
        let times = Arc::clone(&distances);
        let costs = Arc::clone(&distances);
        TravelMatrices {
            distances,
            times,
            costs,
            num_locations,
            is_symmetric: true,
        }
    }

    #[inline(always)]
    pub fn travel_distance(&self, from: usize, to: usize) -> Distance {
        if from == to {
            return 0.0;
        }

        self.distances[self.get_index(from, to)]
    }

    #[inline(always)]
    pub fn travel_time(&self, from: usize, to: usize) -> SignedDuration {
        if from == to {
            return SignedDuration::ZERO;
        }

        SignedDuration::from_secs_f64(self.times[self.get_index(from, to)])
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: usize, to: usize) -> Cost {
        if from == to {
            return 0.0;
        }

        self.costs[self.get_index(from, to)]
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
