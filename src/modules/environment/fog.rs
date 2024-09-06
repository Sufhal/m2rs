use super::Color;

pub struct Fog {
    near: f32,
    far: f32,
    color: Color,
    day: Color,
    night: Color,
}

impl Fog {
    pub fn new(near: f32, far: f32, color: Color) -> Self {
        Self {
            near,
            far,
            color,
            day: color,
            night: color
        }
    }
}