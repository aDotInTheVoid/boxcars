use std::mem;

pub struct SimpleRand {
    value: u64,
}

impl SimpleRand {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
    fn next(&mut self) -> u64 {
        self.next_long()
    }
    fn next_long(&mut self) -> u64 {
        let next = ((self.value * 1309) + 13849) & 65535;
        return mem::replace(&mut self.value, next);
    }
    pub fn next_int(&mut self) -> u32 {
        self.next_int_with_max(0)
    }
    pub fn next_int_with_max(&mut self, max: u32) -> u32 {
        if max == 0 {
            self.next_long() as u32
        } else {
            self.next_long() as u32 % max
        }
    }
    pub fn next_double(&mut self) -> f64 {
        return 1.0 / (self.next_long() + 1) as f64;
    }
}
