use std::ops::Add;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Capacity(Vec<f64>);

impl Capacity {
    pub fn new(capacity: Vec<f64>) -> Self {
        Capacity(capacity)
    }

    pub const ZERO: Capacity = Capacity(vec![]);

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

    fn test_add_op() {
        let capacity1 = Capacity::new(vec![1.0, 2.0, 3.0]);
        let capacity2 = Capacity::new(vec![4.0, 5.0, 6.0]);

        let result = &capacity1 + &capacity2;

        assert_eq!(result, Capacity::new(vec![5.0, 7.0, 9.0]));
    }
}
