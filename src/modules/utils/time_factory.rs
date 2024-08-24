pub struct TimeFactory {
    last_instant: web_time::Instant,
    delta: f64
}

impl TimeFactory {
    pub fn new() -> Self {
        Self { last_instant: web_time::Instant::now(), delta: 0.0 }
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
    pub fn tick(&mut self) {
        let instant = web_time::Instant::now();
        self.delta = instant.duration_since(self.last_instant).as_nanos() as f64 / 1000000.0;
        self.last_instant = instant;
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