use rand::RngCore;

pub struct MockRng {
    data: Vec<u64>,
    index: usize,
}

impl MockRng {
    pub fn new(data: Vec<u64>) -> Self {
        MockRng { data, index: 0 }
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        let value = self.data[self.index % self.data.len()];
        self.index = (self.index + 1) % self.data.len();
        value
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        for byte in dst.iter_mut() {
            *byte = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn test_mock_rng() {
        let data = vec![1, 2, 3, 4];
        let mut rng = MockRng::new(data.clone());

        for &expected in data.iter().cycle().take(8) {
            let value = rng.next_u64();
            assert_eq!(value, expected);
        }
    }

    #[test]
    fn test_random_bool() {
        let data = vec![
            (u64::MAX / 4),
            (u64::MAX / 4),
            (u64::MAX / 4),
            (u64::MAX / 4),
        ];
        let mut rng = MockRng::new(data);

        assert!(!rng.random_bool(0.20));
        assert!(rng.random_bool(0.26));
        assert!(rng.random_bool(0.6));
        assert!(!rng.random_bool(0.10));
    }
}
