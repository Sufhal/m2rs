use cgmath::{Deg, Matrix4, Quaternion, Rotation3, Vector3};

use crate::modules::{core::model::{CustomMesh, TransformUniform}, geometry::plane::Plane, pipelines::sun_pipeline::{self, SunPipeline}, state::State, utils::functions::f32x3};
use super::{cycle::Cycle, environment::MsEnv, Color, Position};

// const DAY_POSITION: [f32; 3] = [376.0, 182.0, 641.0];
const DAY_POSITION: [f32; 3] = [2600.0, 000.0, -1000.0];
const NIGHT_POSITION: [f32; 3] = [1400.0, 1400.0, 1400.0];

pub struct Sun {
    position: Position,
    pub uniform: SunUniform,
    pub mesh: CustomMesh,
}

impl Sun {
    pub fn new(msenv: &MsEnv, state: &State<'_>) -> Self {
        let position = DAY_POSITION;
        let plane = Plane::new(500.0, 500.0, 1, 1);
        let mut uniform = SunUniform::default();
        uniform.material_diffuse = msenv.material.diffuse;
        uniform.material_ambient = msenv.material.ambient;
        uniform.background_diffuse = msenv.directional_light.background.diffuse;
        uniform.background_ambient = msenv.directional_light.background.ambient;
        uniform.character_diffuse = msenv.directional_light.character.diffuse;
        uniform.character_ambient = msenv.directional_light.character.ambient;
        let mesh = plane.to_sun_mesh(
            &state.device, 
            &state.queue, 
            &state.sun_pipeline,
            position,
        );
        Self {
            position,
            uniform,
            mesh,
        }
    } 

    pub fn update(&mut self, cycle: &Cycle, queue: &wgpu::Queue) {
        if cycle.night_factor > 0.0 {
            self.position = NIGHT_POSITION;
            self.uniform.moon_position = [self.position[0], self.position[1], self.position[2], 0.0];
        }
        if cycle.day_factor > 0.0 {
            let angle = 180.0 * cycle.day_factor;
            self.position = (Quaternion::from_axis_angle(Vector3::unit_z(), Deg(angle)) * Vector3::from(DAY_POSITION)).into();
            self.uniform.sun_position = [self.position[0], self.position[1], self.position[2], 0.0];
        }
        queue.write_buffer(
            &self.mesh.transform_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&[self.transform_uniform()]),
        );
    }

    pub fn sun_uniform(&self) -> SunUniform {
        self.uniform
    }

    pub fn transform_uniform(&self) -> TransformUniform {
        TransformUniform::from(Matrix4::from_translation(self.position.into()).into())
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct SunUniform {
    pub sun_position: [f32; 4],
    pub moon_position: [f32; 4],
    pub material_diffuse: [f32; 4],
    pub material_ambient: [f32; 4],
    pub material_emissive: [f32; 4],
    pub background_diffuse: [f32; 4],
    pub background_ambient: [f32; 4],
    pub character_diffuse: [f32; 4],
    pub character_ambient: [f32; 4],
}

impl Default for SunUniform {
    fn default() -> Self {
        Self {
            sun_position: [DAY_POSITION[0], DAY_POSITION[1], DAY_POSITION[2], 0.0],
            moon_position: [NIGHT_POSITION[0], NIGHT_POSITION[1], NIGHT_POSITION[2], 0.0],
            material_diffuse: Default::default(),
            material_ambient: Default::default(),
            material_emissive: Default::default(),
            background_diffuse: Default::default(),
            background_ambient: Default::default(),
            character_diffuse: Default::default(),
            character_ambient: Default::default(),
        }
    }
}