use cgmath::{Deg, Matrix4, Quaternion, Rotation3, Vector3};

use crate::modules::{core::model::{CustomMesh, TransformUniform}, geometry::plane::Plane, pipelines::sun_pipeline::{self, SunPipeline}, state::State};
use super::{cycle::Cycle, Color, Position};

const DAY_POSITION: [f32; 3] = [2600.0, 000.0, -1000.0];
const NIGHT_POSITION: [f32; 3] = [1400.0, 500.0, 1400.0];

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
        let position = DAY_POSITION;
        let plane = Plane::new(500.0, 500.0, 1, 1);
        let mesh = plane.to_sun_mesh(
            &state.device, 
            &state.queue, 
            &state.sun_pipeline,
            position, 
            ambient
        );
        Self {
            diffuse,
            ambient,
            emissive,
            position,
            day: Default::default(),
            night: Default::default(),
            mesh,
        }
    } 

    pub fn update(&mut self, cycle: &Cycle, queue: &wgpu::Queue) {
        if cycle.night_factor > 0.0 {
            self.position = NIGHT_POSITION;
        }
        else {
            let angle = 180.0 * cycle.day_factor;
            self.position = (Quaternion::from_axis_angle(Vector3::unit_z(), Deg(angle)) * Vector3::from(DAY_POSITION)).into();
        }
        queue.write_buffer(
            &self.mesh.transform_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&[self.transform_uniform()]),
        );
    }

    pub fn sun_uniform(&self) -> SunUniform {
        SunUniform {
            color: [
                self.ambient[0],
                self.ambient[1],
                self.ambient[2],
                0.0,
            ]
        }
    }

    pub fn transform_uniform(&self) -> TransformUniform {
        TransformUniform::from(Matrix4::from_translation(self.position.into()).into())
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