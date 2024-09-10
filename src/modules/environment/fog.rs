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
}

impl Fog {
    pub fn new(day_msenv: &MsEnv, night_msenv: &MsEnv) -> Self {
        let s = Self {
            day_near: day_msenv.fog.near * DISTANCE_MULTIPLIER,
            day_far: day_msenv.fog.far * DISTANCE_MULTIPLIER,
            day_color: day_msenv.fog.color,
            night_near: night_msenv.fog.near * DISTANCE_MULTIPLIER,
            night_far: night_msenv.fog.far * DISTANCE_MULTIPLIER,
            night_color: night_msenv.fog.color,
        };
        dbg!(&s);
        s
    }

    pub fn uniform(&self) -> FogUniform {
        FogUniform { 
            day_near: self.day_near, 
            day_far: self.day_far, 
            day_color: self.day_color, 
            night_near: self.night_near, 
            night_far: self.night_far, 
            night_color: self.night_color, 
            padding: Default::default()
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct FogUniform {
    pub day_near: f32,
    pub day_far: f32,
    pub day_color: [f32; 4],
    pub night_near: f32,
    pub night_far: f32,
    pub night_color: [f32; 4],
    padding: [f32; 4],
}

impl Default for FogUniform {
    fn default() -> Self {
        Self { 
            day_near: 0.0, 
            day_far: 0.0, 
            day_color: Default::default(), 
            night_near: 0.0, 
            night_far: 0.0, 
            night_color: Default::default(), 
            padding: Default::default()
        }
    }
}