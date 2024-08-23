pub struct TimeFactory {
    last_tick: f64,
    delta: f64
}

impl TimeFactory {
    pub fn new() -> Self {
        Self { last_tick: Self::from_epoch_to_now(), delta: 0.0 }
    }
    /// Serves the last tick registered
    pub fn now(&self) -> f64 {
        self.last_tick
    }
    /// Returns the milliseconds elapsed sice UNIX_EPOCH
    pub fn from_epoch_to_now() -> f64 {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                return instant::now()
            } else {
                return std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as f64
            }
        }
    }
    /// Save the current timestamp and compute the delta between two last ticks
    pub fn tick(&mut self) {
        let tick = Self::from_epoch_to_now();
        self.delta = tick - self.last_tick;
        self.last_tick = tick;
    }
    /// Get the delta between two last ticks
    pub fn get_delta(&self) -> f64 {
        self.delta
    }
}

pub struct Instant {
    instant: f64
}

impl Instant {
    pub fn now() -> Self {
        Self { instant: TimeFactory::from_epoch_to_now() }
    }
    pub fn duration(&self) -> f64 {
        let now = TimeFactory::from_epoch_to_now();
        now - self.instant
    }
}