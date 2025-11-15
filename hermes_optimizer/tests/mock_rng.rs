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
