use super::environment::MsEnv;

const DISTANCE_MULTIPLIER: f32 = 1.0;

#[derive(Debug)]
pub struct Fog {
    day_near: f32,
    day_far: f32,
    day_color: [f32; 4],
    night_near: f32,
    night_far: f32,
    night_color: [f32; 4],
    enabled: bool,
}

impl Fog {
    pub fn new(day_msenv: &MsEnv, night_msenv: &MsEnv) -> Self {
        Self {
            day_near: day_msenv.fog.near * DISTANCE_MULTIPLIER,
            day_far: day_msenv.fog.far * DISTANCE_MULTIPLIER,
            day_color: day_msenv.fog.color,
            night_near: night_msenv.fog.near * DISTANCE_MULTIPLIER,
            night_far: night_msenv.fog.far * DISTANCE_MULTIPLIER,
            night_color: night_msenv.fog.color,
            enabled: true,
        }
    }

    pub fn uniform(&self) -> FogUniform {
        FogUniform { 
            day_color: self.day_color, 
            padding1: 0.0,
            day_near: self.day_near, 
            padding2: 0.0,
            day_far: self.day_far, 
            night_color: self.night_color, 
            padding3: 0.0,
            night_near: self.night_near, 
            night_far: self.night_far, 
            enabled: if self.enabled { 1.0 } else { 0.0 },
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct FogUniform {
    pub day_color: [f32; 4],
    pub day_near: f32,
    pub day_far: f32,
    pub padding1: f32,
    pub padding2: f32,
    pub night_color: [f32; 4],
    pub night_near: f32,
    pub night_far: f32,
    pub padding3: f32,
    pub enabled: f32,
}

impl Default for FogUniform {
    fn default() -> Self {
        Self { 
            day_near: Default::default(), 
            day_far: Default::default(), 
            padding2: Default::default(),
            padding1: Default::default(),
            day_color: Default::default(),
            night_color: Default::default(),
            night_near: Default::default(),
            night_far: Default::default(),
            padding3: Default::default(),
            enabled: Default::default(),
        }
    }
}