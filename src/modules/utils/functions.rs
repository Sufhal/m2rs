use super::time_factory::TimeFactory;

pub fn debug_using_trash_file(
    name: &str,
    content: String,
) {
    let _ = std::fs::write(
        std::path::Path::new(&format!("trash/{name}.txt")), 
        content
    );
}

pub fn is_browser() -> bool {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            return true;
        } 
        else {
            return false;
        }
    }
}

pub fn normalize_f64(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

pub fn denormalize_f64(value: f64, min: f64, max: f64) -> f64 {
    value * (max - min) + min
}

pub fn denormalize_f32(value: f32, min: f32, max: f32) -> f32 {
    value * (max - min) + min
}

pub fn denormalize_f32x3(value: f32, min: &[f32; 3], max: &[f32; 3]) -> [f32; 3] {
    [
        denormalize_f32(value, min[0], max[0]),
        denormalize_f32(value, min[1], max[1]),
        denormalize_f32(value, min[2], max[2]),
    ]
}

pub fn denormalize_f32x4(value: f32, min: &[f32; 4], max: &[f32; 4]) -> [f32; 4] {
    [
        denormalize_f32(value, min[0], max[0]),
        denormalize_f32(value, min[1], max[1]),
        denormalize_f32(value, min[2], max[2]),
        denormalize_f32(value, min[3], max[3]),
    ]
}

pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    f64::min(f64::max(value, min), max)
}

pub fn mean_f64(data: &Vec<f64>) -> f64 {
    data.iter().fold(0.0, |acc, cur| acc + cur) / data.len() as f64
}

pub fn median_f64(data: &Vec<f64>) -> f64 {
    if data.len() % 2 == 0 {
        mean_f64(&vec![data[data.len() / 2 - 1], data[data.len() / 2]])
    } else {
        data[data.len() / 2]
    }
}

/// Generates a random f64 based on timestamp.
/// If you need different number on the same tick, this function wont help you.
pub fn random_f64_timestamp(max: f64) -> f64 {
    let timestamp = TimeFactory::from_epoch_to_now();
    timestamp % max
}

pub fn random_u8(max: u8) -> u8 {
    fastrand::u8(0..max)
}

pub fn is_power_of_two(x: u64) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}

pub fn u8_to_string_with_len(value: u8, len: usize) -> String {
    let mut output = format!("{value}");
    while output.len() < len {
        output.insert_str(0, "0");
    }
    output
}

pub fn calculate_fps(frame_time_ms: f64) -> f64 {
    if frame_time_ms == 0.0 {
        0.0
    } else {
        1000.0 / frame_time_ms
    }
}

pub fn to_fixed_2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}