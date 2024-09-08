use crate::modules::utils::functions::srgb_to_linear;
use super::Color;

pub struct Fog {
    near: f32,
    far: f32,
    color: Color,
}

impl Fog {
    pub fn new(near: f32, far: f32, color: Color) -> Self {
        Self {
            near: near / 100.0,
            far: far / 100.0,
            color: srgb_to_linear(color),
        }
    }

    pub fn uniform(&self) -> FogUniform {
        FogUniform { 
            near: self.near, 
            padding1: 0.0,
            far: self.far, 
            padding2: 0.0,
            color: self.color, 
            padding3: 0.0,
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct FogUniform {
    pub near: f32,
    pub padding1: f32,
    pub far: f32,
    pub padding2: f32,
    pub color: [f32; 3],
    pub padding3: f32,
}

impl Default for FogUniform {
    fn default() -> Self {
        Self { 
            near: 0.0, 
            padding1: 0.0,
            far: 0.0, 
            padding2: 0.0,
            color: Default::default(), 
            padding3: 0.0,
        }
    }
}