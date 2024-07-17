pub struct IdGen {
    current: u32
}

impl IdGen {
    pub fn new() -> Self {
        Self { current: 0 }
    }
    pub fn generate(&mut self) -> u32 {
        let current = self.current;
        self.current += 1;
        current
    }
}