use super::Color;

pub struct Fog {
    near: f32,
    far: f32,
    color: Color,
    day: Color,
    night: Color,
}