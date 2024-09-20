use std::f32::consts::PI;

use cgmath::{Rad, Vector3};
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

pub fn normalize_f32(value: f32, min: f32, max: f32) -> f32 {
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

pub fn f32x3(array: &[f32; 4]) -> [f32; 3] {
    [array[0], array[1], array[2]]
}

pub fn add_normals(normal: [f32; 3], additional_normal: Vector3<f32>) -> [f32; 3] {
    [
        normal[0] + additional_normal.x,
        normal[1] + additional_normal.y,
        normal[2] + additional_normal.z,
    ]
}

pub fn srgb_to_linear(color: [f32; 3]) -> [f32; 3] {
    let r = if color[0] <= 0.04045 {
        color[0] / 12.92
    } else {
        ((color[0] + 0.055) / 1.055).powf(2.4)
    };

    let g = if color[1] <= 0.04045 {
        color[1] / 12.92
    } else {
        ((color[1] + 0.055) / 1.055).powf(2.4)
    };

    let b = if color[2] <= 0.04045 {
        color[2] / 12.92
    } else {
        ((color[2] + 0.055) / 1.055).powf(2.4)
    };

    [r, g, b]
}

pub fn correct_color(colors: [f32; 4]) -> [f32; 4] {
    let rgb = srgb_to_linear([colors[0], colors[1], colors[2]]);
    [rgb[0], rgb[1], rgb[2], colors[3]]
}

pub fn lerp_angle(start: Rad<f32>, end: Rad<f32>, t: f32) -> Rad<f32> {
    let mut delta = (end - start).0;
    if delta > PI {
        delta -= 2.0 * PI;
    } else if delta < -PI {
        delta += 2.0 * PI;
    }
    Rad(start.0 + delta * t.clamp(0.0, 1.0))
}
