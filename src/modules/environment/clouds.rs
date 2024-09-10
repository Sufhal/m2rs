use crate::modules::{assets::assets::load_texture, core::{model::CustomMesh, texture::Texture}, geometry::plane::Plane, state::State};
use super::environment::MsEnv;

const SIZE_MULTIPLIER: f32 = 2.0;

pub struct Clouds {
    texture: Texture,
    uniform: CloudsUniform,
    buffer: wgpu::Buffer,
    pub mesh: CustomMesh,
}

impl Clouds {
    pub async fn new(msenv: &MsEnv, state: &State<'_>) -> anyhow::Result<Self> {
        let texture = load_texture(
            &msenv.sky_box.cloud_texture_file, 
            &state.device, 
            &state.queue
        ).await?;
        let uniform = CloudsUniform {
            time: 0.0,
            speed: msenv.sky_box.cloud_speed[0],
            scale: [
                msenv.sky_box.cloud_texture_scale[0] * SIZE_MULTIPLIER,
                msenv.sky_box.cloud_texture_scale[1] * SIZE_MULTIPLIER,
            ],
        };
        let plane = Plane::new(
            msenv.sky_box.cloud_scale[0] * SIZE_MULTIPLIER, 
            msenv.sky_box.cloud_scale[1] * SIZE_MULTIPLIER, 
            1, 
            1
        );
        let (mesh, buffer) = plane.to_clouds_mesh(
            &state.device, 
            &state.clouds_pipeline, 
            [
                0.0,
                msenv.sky_box.cloud_height, 
                0.0,
            ], 
            &texture, 
            uniform.clone()
        );
        Ok(Self {
            texture,
            uniform,
            mesh,
            buffer,
        })
    }

    pub fn update(&mut self, delta_ms: f32, queue: &wgpu::Queue) {
        self.uniform.time += delta_ms / (1000.0 / SIZE_MULTIPLIER);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
pub struct CloudsUniform {
    pub time: f32,
    pub speed: f32,
    pub scale: [f32; 2],
}