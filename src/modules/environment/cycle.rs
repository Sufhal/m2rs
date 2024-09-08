use crate::modules::utils::functions::{denormalize_f32, normalize_f32};

const REAL_TIME: bool = false; 
const MS_IN_DAY: u64 = 24 * 3600 * 1000;
const UTC: u64 = 2;
const DAY_START_HOUR: u64 = 5;
const DAY_END_HOUR: u64 = 23;
const DAY_RANGE_MS: [f32; 2] = [
    (DAY_START_HOUR * 60 * 60 * 1000) as f32, 
    (DAY_END_HOUR * 60 * 60 * 1000) as f32
];
const NIGHT_EARLY_RANGE_MS: [f32; 2] = [
    DAY_RANGE_MS[1], 
    (24 * 60 * 60 * 1000) as f32
];
const NIGHT_LATE_RANGE_MS: [f32; 2] = [
    0.0,
    DAY_RANGE_MS[0], 
];
const DAY_DURATION: f32 = DAY_END_HOUR as f32 - DAY_START_HOUR as f32;
const NIGHT_DURATION: f32 = 24.0 - DAY_DURATION;
const NIGHT_EARLY_DURATION: f32 = (NIGHT_EARLY_RANGE_MS[1] - NIGHT_EARLY_RANGE_MS[0]) / (1000.0 * 60.0 * 60.0);
const NIGHT_LATE_DURATION: f32 = NIGHT_DURATION - NIGHT_EARLY_DURATION;
const NIGHT_EARLY_RATIO: f32 = NIGHT_EARLY_DURATION / NIGHT_LATE_DURATION;

pub struct Cycle {
    pub day_factor: f32,
    pub night_factor: f32,
    todays_ms: f32,
}

impl Cycle {
    pub fn new() -> Self {
        let todays_ms = Self::elapsed_ms_today() as f32;
        Self {
            day_factor: Self::compute_day_factor(todays_ms),
            night_factor: Self::compute_night_factor(todays_ms),
            todays_ms
        }
    }

    pub fn update(&mut self, mut delta: f32) {
        if REAL_TIME == false {
            delta *= (86_400_000.0 / 60_000.0) * 4.0; // 24h in 1min
            // delta *= 86_400_000.0 / 60_000.0; // 24h in 1min
        }
        self.todays_ms += delta;
        if self.todays_ms > MS_IN_DAY as f32 {
            self.todays_ms = 0.0;
        }
        self.day_factor = Self::compute_day_factor(self.todays_ms);
        self.night_factor = Self::compute_night_factor(self.todays_ms);
        // dbg!(self);
    } 

    pub fn get_current_time(&self) -> (f32, f32) {
        let total_seconds = self.todays_ms / 1000.0;
        let hours = total_seconds / 3600.0;
        let minutes = (total_seconds % 3600.0) / 60.0;
        (hours, minutes)
    }

    fn elapsed_ms_today() -> u64 {
        let now = web_time::SystemTime::now().duration_since(web_time::UNIX_EPOCH).unwrap();
        let total_seconds = now.as_secs() + (UTC * 60 * 60);
        let seconds_in_day = 24 * 3600;
        let seconds_since_midnight = total_seconds % seconds_in_day;
        let millis = now.subsec_millis() as u64;
        (seconds_since_midnight * 1000) + millis
    }

    fn compute_day_factor(todays_ms: f32) -> f32 {
        if todays_ms <= DAY_RANGE_MS[0] || todays_ms > DAY_RANGE_MS[1] {
            return 0.0
        }
        else {
            normalize_f32(todays_ms, DAY_RANGE_MS[0], DAY_RANGE_MS[1])
        }
    } 

    fn compute_night_factor(todays_ms: f32) -> f32 {
        if todays_ms >= DAY_RANGE_MS[0] && todays_ms < DAY_RANGE_MS[1] {
            return 0.0
        }
        else if todays_ms >= DAY_RANGE_MS[1] { // early night
            let factor = normalize_f32(todays_ms, NIGHT_EARLY_RANGE_MS[0], NIGHT_EARLY_RANGE_MS[1]);
            denormalize_f32(factor, 0.0, NIGHT_EARLY_RATIO)
        }
        else { // late night
            let factor = normalize_f32(todays_ms, NIGHT_LATE_RANGE_MS[0], NIGHT_LATE_RANGE_MS[1]);
            denormalize_f32(factor, NIGHT_EARLY_RATIO, 1.0)
        }
    } 
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct CycleUniform {
    pub day_factor: f32,
    pub night_factor: f32,
}

impl Default for CycleUniform {
    fn default() -> Self {
        Self {
            day_factor: 0.0,
            night_factor: 0.0,
        }
    }
}