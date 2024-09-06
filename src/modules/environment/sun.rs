use crate::modules::{core::model::CustomMesh, geometry::plane::Plane, state::State};
use super::{Color, Position};

pub struct Sun {
    position: Position,
    diffuse: Color,
    ambient: Color,
    emissive: Color,
    day: [Color; 3],
    night: [Color; 3],
    pub mesh: CustomMesh
}

impl Sun {
    pub fn new(diffuse: Color, ambient: Color, emissive: Color, state: &State<'_>) -> Self {
        let plane = Plane::new(500.0, 500.0, 1, 1);
        let mesh = plane.to_sun_mesh(
            &state.device, 
            &state.queue, 
            &state.sun_pipeline,
            [2600.0, 400.0, -1000.0], 
            ambient
        );
        Self {
            diffuse,
            ambient,
            emissive,
            position: Default::default(),
            day: Default::default(),
            night: Default::default(),
            mesh,
        }
    } 

    pub fn uniform(&self) -> SunUniform {
        SunUniform {
            color: [
                self.ambient[0],
                self.ambient[1],
                self.ambient[2],
                0.0,
            ]
        }
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct SunUniform {
    color: [f32; 4]
}

impl SunUniform {
    pub fn new(color: Color) -> Self {
        Self {
            color: [
                color[0],
                color[1],
                color[2],
                0.0,
            ]
        }
    }
}