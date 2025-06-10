pub type Distance = f64;
pub type Time = usize;
pub type Cost = usize;

pub struct TravelCostMatrix {
    distances: Vec<Vec<Distance>>,
    times: Vec<Vec<Time>>,
    costs: Vec<Vec<Cost>>,
}

impl TravelCostMatrix {
    #[inline(always)]
    pub fn get_distance(&self, from: usize, to: usize) -> Distance {
        self.distances[from][to]
    }

    #[inline(always)]
    pub fn get_time(&self, from: usize, to: usize) -> Time {
        self.times[from][to]
    }

    #[inline(always)]
    pub fn get_cost(&self, from: usize, to: usize) -> Cost {
        self.costs[from][to]
    }
}
