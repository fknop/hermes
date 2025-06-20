#[derive(Default, Debug, PartialEq)]
pub struct Capacity(Vec<f64>);

impl Capacity {
    pub fn new(capacity: Vec<f64>) -> Self {
        Capacity(capacity)
    }

    pub fn reset(&mut self) {
        self.0.fill(0.0);
    }

    pub fn add(&mut self, other: &Capacity) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0.0);
        }

        for i in 0..other.0.len() {
            self.0[i] += other.0[i];
        }
    }

    pub fn sub(&mut self, other: &Capacity) {
        if self.0.len() < other.0.len() {
            self.0.resize(other.0.len(), 0.0);
        }

        for i in 0..other.0.len() {
            self.0[i] -= other.0[i];
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add() {
        let mut total_capacity = Capacity::default();

        total_capacity.add(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![1.0, 2.0, 3.0]));

        total_capacity.add(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![2.0, 4.0, 6.0]));
    }

    #[test]
    fn test_sub() {
        let mut total_capacity = Capacity::new(vec![10.0, 4.0, 5.0]);

        total_capacity.sub(&Capacity(vec![1.0, 2.0, 3.0]));

        assert_eq!(total_capacity, Capacity::new(vec![9.0, 2.0, 2.0]));

        total_capacity.sub(&Capacity(vec![1.0, 0.0, 0.0]));

        assert_eq!(total_capacity, Capacity::new(vec![8.0, 2.0, 2.0]));
    }
}
