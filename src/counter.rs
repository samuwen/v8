#[derive(Debug, Clone)]
pub struct Counter(u32);

impl Counter {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn get(&mut self) -> u32 {
        let current = self.0;
        self.0 += 1;
        current
    }
}
