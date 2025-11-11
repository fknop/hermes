use std::ops::{Add, AddAssign, Index, Sub, SubAssign};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::utils::normalize::normalize;

type CapacityVector = SmallVec<[f64; 4]>;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Capacity(CapacityVector);

impl Capacity {
    pub fn from_vec(vec: Vec<f64>) -> Self {
        Capacity(CapacityVector::from_vec(vec))
    }

    pub fn new(capacity: CapacityVector) -> Self {
        Capacity(capacity)
    }

    pub const ZERO: Capacity = Capacity(CapacityVector::new_const());

    pub fn zero() -> Self {
        Capacity(CapacityVector::new())
    }

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
        let mut min = CapacityVector::with_capacity(max_size);
        min.resize(max_size, 0.0);

        let mut max = CapacityVector::with_capacity(max_size);
        max.resize(max_size, 0.0);

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

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() || self.0.iter().all(|&c| c == 0.0)
    }

    pub fn reset(&mut self) {
        self.0.fill(0.0);
    }

    pub fn satisfies_demand(&self, demand: &Capacity) -> bool {
        if self.len() < demand.len() {
            return false;
        }

        demand.0.iter().zip(self.0.iter()).all(|(d, c)| d <= c)
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

impl AddAssign<&Capacity> for Capacity {
    fn add_assign(&mut self, rhs: &Capacity) {
        if self.0.len() < rhs.0.len() {
            self.0.resize(rhs.0.len(), 0.0);
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a += *b;
        }
    }
}

impl SubAssign<&Capacity> for Capacity {
    fn sub_assign(&mut self, rhs: &Capacity) {
        if self.0.len() < rhs.0.len() {
            self.0.resize(rhs.0.len(), 0.0);
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a -= *b;
        }
    }
}

impl Add<&Capacity> for &Capacity {
    type Output = Capacity;

    fn add(self, rhs: &Capacity) -> Self::Output {
        if self.len() == rhs.len() {
            let mut output = CapacityVector::new();

            for (a, b) in self.0.iter().zip(rhs.iter()) {
                output.push(a + b);
            }
            Capacity(output)
        } else {
            let mut output = self.clone();

            output += rhs;

            output
        }
    }
}

impl Sub<&Capacity> for &Capacity {
    type Output = Capacity;

    fn sub(self, rhs: &Capacity) -> Self::Output {
        if self.len() == rhs.len() {
            let mut output = CapacityVector::new();

            for (a, b) in self.0.iter().zip(rhs.iter()) {
                output.push(a - b);
            }
            Capacity(output)
        } else {
            let mut output = self.clone();

            output -= rhs;

            output
        }
    }
}

impl Index<usize> for Capacity {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(test)]
mod tests {

    use std::cmp::Ordering;

    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_add_mut() {
        let mut total_capacity = Capacity::default();

        total_capacity.add_assign(&Capacity::from_vec(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::from_vec(vec![1.0, 2.0, 3.0]));

        total_capacity.add_assign(&Capacity::from_vec(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::from_vec(vec![2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_sub_mut() {
        let mut total_capacity = Capacity::from_vec(vec![10.0, 4.0, 5.0]);

        total_capacity.sub_assign(&Capacity::from_vec(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::from_vec(vec![9.0, 2.0, 2.0]));

        total_capacity.sub_assign(&Capacity::from_vec(vec![1.0, 0.0, 0.0]));

        assert_eq!(total_capacity, Capacity::from_vec(vec![8.0, 2.0, 2.0]));
    }

    #[test]
    fn satisfies_demand() {
        let total_capacity = Capacity::from_vec(vec![10.0, 5.0, 8.0]);
        let demand = Capacity::from_vec(vec![5.0, 3.0, 2.0]);

        assert!(total_capacity.satisfies_demand(&demand));

        let insufficient_demand = Capacity::from_vec(vec![11.0, 6.0, 2.0]);

        assert!(!total_capacity.satisfies_demand(&insufficient_demand));
    }

    #[test]
    pub fn over_capacity_demand() {
        let total_capacity = Capacity::from_vec(vec![10.0, 5.0, 8.0, 5.0]);
        let demand = Capacity::from_vec(vec![5.0, 3.0, 2.0, 8.0]);

        assert_eq!(total_capacity.over_capacity_demand(&demand), 3.0);
        assert_eq!(demand.over_capacity_demand(&total_capacity), 13.0);
    }

    #[test]
    fn test_add_op() {
        let capacity1 = Capacity::from_vec(vec![1.0, 2.0, 3.0]);
        let capacity2 = Capacity::from_vec(vec![4.0, 5.0, 6.0]);

        let result = &capacity1 + &capacity2;

        assert_eq!(result, Capacity::from_vec(vec![5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_min_max_capacities() {
        let capacities = [
            Capacity::from_vec(vec![1.0, 2.0, 3.0]),
            Capacity::from_vec(vec![4.0, 5.0, 6.0]),
            Capacity::from_vec(vec![2.0, 3.0, 4.0]),
        ];

        let (min, max) =
            Capacity::compute_min_max_capacities(&(capacities.iter().collect::<Vec<&Capacity>>()));

        assert_eq!(min, Capacity::from_vec(vec![1.0, 2.0, 3.0]));
        assert_eq!(max, Capacity::from_vec(vec![4.0, 5.0, 6.0]));
    }

    #[test]
    fn test_normalize() {
        let capacity = Capacity::from_vec(vec![5.0, 10.0, 15.0]);
        let min = Capacity::from_vec(vec![0.0, 5.0, 10.0]);
        let max = Capacity::from_vec(vec![10.0, 15.0, 20.0]);

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
