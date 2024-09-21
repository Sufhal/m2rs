use cgmath::{Angle, Matrix4, Rad};
use crate::modules::{core::model::{CustomMesh, TransformUniform}, geometry::plane::Plane, state::State};
use super::{cycle::Cycle, environment::MsEnv, Position};

const DAY_POSITION: [f32; 3] = [2600.0, 000.0, -1000.0];
const NIGHT_POSITION: [f32; 3] = [1400.0, 1400.0, 1400.0];

pub struct Sun {
    center: [f32; 3],
    position: Position,
    pub uniform: SunUniform,
    pub mesh: CustomMesh,
}

impl Sun {
    pub fn new(day_msenv: &MsEnv, night_msenv: &MsEnv, center: [f32; 3], state: &State<'_>) -> Self {
        let position = DAY_POSITION;
        let plane = Plane::new(500.0, 500.0, 1, 1);
        let mut uniform = SunUniform::default();
        uniform.day_material_diffuse = day_msenv.material.diffuse;
        uniform.day_material_ambient = day_msenv.material.ambient;
        uniform.day_background_diffuse = day_msenv.directional_light.background.diffuse;
        uniform.day_background_ambient = day_msenv.directional_light.background.ambient;
        uniform.day_character_diffuse = day_msenv.directional_light.character.diffuse;
        uniform.day_character_ambient = day_msenv.directional_light.character.ambient;
        uniform.night_material_diffuse = night_msenv.material.diffuse;
        uniform.night_material_ambient = night_msenv.material.ambient;
        uniform.night_background_diffuse = night_msenv.directional_light.background.diffuse;
        uniform.night_background_ambient = night_msenv.directional_light.background.ambient;
        uniform.night_character_diffuse = night_msenv.directional_light.character.diffuse;
        uniform.night_character_ambient = night_msenv.directional_light.character.ambient;
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
            center,
        }
    } 

    pub fn update(&mut self, cycle: &Cycle, queue: &wgpu::Queue) {
        if cycle.night_factor > 0.0 {
            self.position = NIGHT_POSITION;
            self.uniform.moon_position = [self.position[0], self.position[1], self.position[2], 0.0];
        }
        if cycle.day_factor > 0.0 {
            let angle = 180.0 * cycle.day_factor;
            // self.position = (Quaternion::from_axis_angle(Vector3::unit_z(), Deg(angle)) * Vector3::from(DAY_POSITION)).into();
            self.position = compute_sun_position(&self.center, 2000.0, angle);
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
    pub day_material_diffuse: [f32; 4],
    pub day_material_ambient: [f32; 4],
    pub day_material_emissive: [f32; 4],
    pub day_background_diffuse: [f32; 4],
    pub day_background_ambient: [f32; 4],
    pub day_character_diffuse: [f32; 4],
    pub day_character_ambient: [f32; 4],
    pub night_material_diffuse: [f32; 4],
    pub night_material_ambient: [f32; 4],
    pub night_material_emissive: [f32; 4],
    pub night_background_diffuse: [f32; 4],
    pub night_background_ambient: [f32; 4],
    pub night_character_diffuse: [f32; 4],
    pub night_character_ambient: [f32; 4],
}

impl Default for SunUniform {
    fn default() -> Self {
        Self {
            sun_position: [DAY_POSITION[0], DAY_POSITION[1], DAY_POSITION[2], 0.0],
            moon_position: [NIGHT_POSITION[0], NIGHT_POSITION[1], NIGHT_POSITION[2], 0.0],
            day_material_diffuse: Default::default(),
            day_material_ambient: Default::default(),
            day_material_emissive: Default::default(),
            day_background_diffuse: Default::default(),
            day_background_ambient: Default::default(),
            day_character_diffuse: Default::default(),
            day_character_ambient: Default::default(),
            night_material_diffuse: Default::default(),
            night_material_ambient: Default::default(),
            night_material_emissive: Default::default(),
            night_background_diffuse: Default::default(),
            night_background_ambient: Default::default(),
            night_character_diffuse: Default::default(),
            night_character_ambient: Default::default(),
        }
    }
}

fn compute_sun_position(center: &[f32; 3], radius: f32, angle_deg: f32) -> [f32; 3] {
    let angle_rad = Rad::from(cgmath::Deg(angle_deg));
    let x = center[0] + radius * angle_rad.cos();
    let z = center[2];
    let y = center[1] + radius * angle_rad.sin();
    [x, y, z]
}