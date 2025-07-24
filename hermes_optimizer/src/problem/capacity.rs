use std::ops::Add;

use serde::{Deserialize, Serialize};

use crate::utils::normalize::normalize;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Capacity(Vec<f64>);

impl Capacity {
    pub fn new(capacity: Vec<f64>) -> Self {
        Capacity(capacity)
    }

    pub const ZERO: Capacity = Capacity(vec![]);

    pub fn iter(&self) -> impl Iterator<Item = f64> {
        self.0.iter().cloned()
    }

    pub fn get(&self, index: usize) -> Option<f64> {
        self.0.get(index).cloned()
    }

    pub fn compute_min_max_capacities(capacities: &[&Capacity]) -> (Capacity, Capacity) {
        if capacities.is_empty() {
            return (Capacity::ZERO, Capacity::ZERO);
        }

        let max_size = capacities.iter().map(|c| c.0.len()).max().unwrap_or(0);
        let mut min = vec![0.0; max_size];
        let mut max = vec![0.0; max_size];

        for i in 0..max_size {
            min[i] = capacities
                .iter()
                .map(|c| c.0.get(i).cloned().unwrap_or(0.0))
                .fold(f64::INFINITY, |a, b| a.min(b));

            max[i] = capacities
                .iter()
                .map(|c| c.0.get(i).cloned().unwrap_or(0.0))
                .fold(0.0_f64, |a, b| a.max(b));
        }

        (Capacity(min), Capacity(max))
    }

    pub fn normalize(&self, min: &Capacity, max: &Capacity) -> Vec<f64> {
        if self.0.is_empty() {
            return vec![];
        }
        let mut normalized = vec![0.0; self.0.len()];

        (0..self.0.len()).for_each(|i| {
            normalized[i] = normalize(self.0[i], min.0[i], max.0[i]);
        });

        normalized
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|&c| c == 0.0)
    }

    pub fn reset(&mut self) {
        self.0.fill(0.0);
    }

    pub fn add_mut(&mut self, other: &Capacity) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0.0);
        }

        for i in 0..other.0.len() {
            self.0[i] += other.0[i];
        }
    }

    pub fn sub_mut(&mut self, other: &Capacity) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0.0);
        }

        for i in 0..other.0.len() {
            self.0[i] -= other.0[i];
        }
    }

    pub fn satisfies_demand(&self, demand: &Capacity) -> bool {
        if self.0.len() < demand.0.len() {
            return false;
        }

        for i in 0..demand.0.len() {
            if self.0[i] < demand.0[i] {
                return false;
            }
        }

        true
    }

    pub fn over_capacity_demand(&self, demand: &Capacity) -> f64 {
        let mut over_capacity = 0.0;

        for i in 0..demand.0.len() {
            if self.0[i] < demand.0[i] {
                over_capacity += demand.0[i] - self.0[i];
            }
        }

        over_capacity
    }
}

impl Add<&Capacity> for &Capacity {
    type Output = Capacity;

    fn add(self, rhs: &Capacity) -> Self::Output {
        let mut output = Capacity::ZERO;

        output.add_mut(self);
        output.add_mut(rhs);

        output
    }
}

#[cfg(test)]
mod tests {

    use std::cmp::Ordering;

    use super::*;

    #[test]
    fn test_add_mut() {
        let mut total_capacity = Capacity::default();

        total_capacity.add_mut(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![1.0, 2.0, 3.0]));

        total_capacity.add_mut(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_sub_mut() {
        let mut total_capacity = Capacity::new(vec![10.0, 4.0, 5.0]);

        total_capacity.sub_mut(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![9.0, 2.0, 2.0]));

        total_capacity.sub_mut(&Capacity(vec![1.0, 0.0, 0.0]));

        assert_eq!(total_capacity, Capacity::new(vec![8.0, 2.0, 2.0]));
    }

    #[test]
    fn satisfies_demand() {
        let total_capacity = Capacity::new(vec![10.0, 5.0, 8.0]);
        let demand = Capacity::new(vec![5.0, 3.0, 2.0]);

        assert!(total_capacity.satisfies_demand(&demand));

        let insufficient_demand = Capacity::new(vec![11.0, 6.0, 2.0]);

        assert!(!total_capacity.satisfies_demand(&insufficient_demand));
    }

    #[test]
    pub fn over_capacity_demand() {
        let total_capacity = Capacity::new(vec![10.0, 5.0, 8.0, 5.0]);
        let demand = Capacity::new(vec![5.0, 3.0, 2.0, 8.0]);

        assert_eq!(total_capacity.over_capacity_demand(&demand), 3.0);
        assert_eq!(demand.over_capacity_demand(&total_capacity), 13.0);
    }

    #[test]
    fn test_add_op() {
        let capacity1 = Capacity::new(vec![1.0, 2.0, 3.0]);
        let capacity2 = Capacity::new(vec![4.0, 5.0, 6.0]);

        let result = &capacity1 + &capacity2;

        assert_eq!(result, Capacity::new(vec![5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_min_max_capacities() {
        let capacities = vec![
            Capacity::new(vec![1.0, 2.0, 3.0]),
            Capacity::new(vec![4.0, 5.0, 6.0]),
            Capacity::new(vec![2.0, 3.0, 4.0]),
        ];

        let (min, max) =
            Capacity::compute_min_max_capacities(&(capacities.iter().collect::<Vec<&Capacity>>()));

        assert_eq!(min, Capacity::new(vec![1.0, 2.0, 3.0]));
        assert_eq!(max, Capacity::new(vec![4.0, 5.0, 6.0]));
    }

    #[test]
    fn test_normalize() {
        let capacity = Capacity::new(vec![5.0, 10.0, 15.0]);
        let min = Capacity::new(vec![0.0, 5.0, 10.0]);
        let max = Capacity::new(vec![10.0, 15.0, 20.0]);

        let normalized = capacity.normalize(&min, &max);

        // 5 - 0 / 10 - 0 = 0.5
        // 10 - 5 / 15 - 5 = 5 / 10 = 0.5
        // 15 - 10 / 20 - 10 = 5 / 10 = 0.5

        assert_eq!(normalized, vec![0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_cmp_vectors() {
        let less = vec![0.5, 0.5, 0.5].partial_cmp(&vec![0.6, 0.6, 0.6]);
        assert_eq!(less, Some(Ordering::Less));

        let more = vec![0.7, 0.6, 0.7].partial_cmp(&vec![0.6, 0.6, 0.6]);
        assert_eq!(more, Some(Ordering::Greater));

        let more = vec![0.6, 0.6, 0.6].partial_cmp(&vec![0.6, 0.6, 0.6]);
        assert_eq!(more, Some(Ordering::Equal));
    }
}
