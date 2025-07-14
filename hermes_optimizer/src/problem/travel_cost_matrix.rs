use serde::Deserialize;

use super::location::Location;

pub type Distance = f64;
pub type Time = i64;
pub type Cost = f64;

#[derive(Deserialize)]
pub struct TravelCostMatrix {
    distances: Vec<Vec<Distance>>,
    times: Vec<Vec<Time>>,
    costs: Vec<Vec<Cost>>,
}

impl TravelCostMatrix {
    pub fn from_haversine(locations: &[Location]) -> Self {
        let mut distances: Vec<Vec<Distance>> = vec![vec![0.0; locations.len()]; locations.len()];
        let mut times: Vec<Vec<Time>> = vec![vec![0; locations.len()]; locations.len()];
        let mut costs: Vec<Vec<Cost>> = vec![vec![0.0; locations.len()]; locations.len()];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i][j] = from.haversine_distance(to);
                // Assume average speed of 50km/h
                let speed = 50.0 / 3.6;
                times[i][j] = (distances[i][j] / speed).round() as Time;
                costs[i][j] = distances[i][j]
            }
        }

        TravelCostMatrix {
            distances,
            times,
            costs,
        }
    }

    pub fn from_euclidian(locations: &[Location]) -> Self {
        let mut distances: Vec<Vec<Distance>> = vec![vec![0.0; locations.len()]; locations.len()];
        let mut times: Vec<Vec<Time>> = vec![vec![0; locations.len()]; locations.len()];
        let mut costs: Vec<Vec<Cost>> = vec![vec![0.0; locations.len()]; locations.len()];

        for (i, from) in locations.iter().enumerate() {
            for (j, to) in locations.iter().enumerate() {
                distances[i][j] = from.euclidian_distance(to);
                times[i][j] = distances[i][j].round() as i64;
                costs[i][j] = distances[i][j]
            }
        }

        TravelCostMatrix {
            distances,
            times,
            costs,
        }
    }

    #[inline(always)]
    pub fn travel_distance(&self, from: usize, to: usize) -> Distance {
        self.distances[from][to]
    }

    #[inline(always)]
    pub fn travel_time(&self, from: usize, to: usize) -> Time {
        self.times[from][to]
    }

    #[inline(always)]
    pub fn travel_cost(&self, from: usize, to: usize) -> Cost {
        self.costs[from][to]
    }
}
