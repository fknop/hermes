use serde::Deserialize;

use super::location::Location;

pub type Distance = f64;
pub type Time = i64;
pub type Cost = f64;

/// This matrix use a flat structure to store distances, times, and costs between locations.
/// To find the index for a pair of locations, use the formula:
/// `index = from * num_locations + to`, where `num_locations` is the total
#[derive(Deserialize)]
pub struct TravelCostMatrix {
    distances: Vec<Distance>,
    times: Vec<Time>,
    costs: Vec<Cost>,
    num_locations: usize,
}

impl TravelCostMatrix {
    pub fn new(
        distances: Vec<Vec<Distance>>,
        times: Vec<Vec<Time>>,
        costs: Vec<Vec<Cost>>,
    ) -> Self {
        let num_locations = distances.len();
        TravelCostMatrix {
            distances: distances.into_iter().flatten().collect(),
            times: times.into_iter().flatten().collect(),
            costs: costs.into_iter().flatten().collect(),
            num_locations,
        }
    }

    #[inline(always)]
    fn get_index(&self, from: usize, to: usize) -> usize {
        from * self.num_locations + to
    }

    pub fn from_haversine(locations: &[Location]) -> Self {
        let num_locations = locations.len();
        let mut distances: Vec<Distance> = vec![0.0; num_locations * num_locations];
        let mut times: Vec<Time> = vec![0; num_locations * num_locations];
        let mut costs: Vec<Cost> = vec![0.0; num_locations * num_locations];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i * num_locations + j] = from.haversine_distance(to);
                // Assume average speed of 50km/h
                let speed = 50.0 / 3.6;
                times[i * num_locations + j] =
                    (distances[i * num_locations + j] / speed).round() as Time;
                costs[i * num_locations + j] = distances[i * num_locations + j]
            }
        }

        TravelCostMatrix {
            distances,
            times,
            costs,
            num_locations,
        }
    }

    pub fn from_euclidian(locations: &[Location]) -> Self {
        let num_locations = locations.len();
        let mut distances: Vec<Distance> = vec![0.0; num_locations * num_locations];
        let mut times: Vec<Time> = vec![0; num_locations * num_locations];
        let mut costs: Vec<Cost> = vec![0.0; num_locations * num_locations];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i * num_locations + j] = from.euclidian_distance(to);
                times[i * num_locations + j] = distances[i * num_locations + j].round() as i64;
                costs[i * num_locations + j] = distances[i * num_locations + j]
            }
        }

        TravelCostMatrix {
            distances,
            times,
            costs,
            num_locations,
        }
    }

    #[inline(always)]
    pub fn travel_distance(&self, from: usize, to: usize) -> Distance {
        self.distances[self.get_index(from, to)]
    }

    #[inline(always)]
    pub fn travel_time(&self, from: usize, to: usize) -> Time {
        self.times[self.get_index(from, to)]
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: usize, to: usize) -> Cost {
        self.costs[self.get_index(from, to)]
    }

    pub fn max_cost(&self) -> Cost {
        self.costs.iter().cloned().fold(0.0, f64::max)
    }
}
