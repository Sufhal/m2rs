use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use crate::modules::camera::camera::OPENGL_TO_WGPU_MATRIX;

use super::texture::Texture;

pub struct DirectionalLight {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub shadow_texture: Texture,
}

impl DirectionalLight {

    pub fn new(position: [f32; 3], target: [f32; 3], device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = Texture { view, sampler, texture };
        
        Self {
            position: position.into(),
            target: target.into(),
            shadow_texture,
        }
    }

    pub fn uniform(&self, aspect: f32) -> DirectionalLightUniform {
        let up = Vector3::unit_y();
        let light_view = Matrix4::look_at_rh(self.position, self.target, up);
        let light_projection = OPENGL_TO_WGPU_MATRIX * perspective(Deg(28.0), aspect, 0.1, 100000.0);
        let mx_view_proj = light_projection * light_view;

        DirectionalLightUniform {
            view_position: self.position.to_homogeneous().into(),
            view_proj: mx_view_proj.into(),
            view_matrix: light_view.into(),
            projection_matrix: light_projection.into(),
        }
    }


}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
}

impl Default for DirectionalLightUniform {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            view_proj: Default::default(),
            view_matrix: Default::default(),
            projection_matrix: Default::default(),
        }
    }
}