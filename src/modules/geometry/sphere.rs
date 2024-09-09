use cgmath::Matrix4;
use wgpu::util::DeviceExt;
use crate::modules::{core::model::{CustomMesh, Mesh, SimpleVertex, TransformUniform}, environment::sky::{Sky, SkyUniform}, pipelines::sky_pipeline::SkyPipeline};
use super::buffer::ToMesh;

pub struct Sphere {
    vertices: Vec<SimpleVertex>,
    indices: Vec<u32>,
}

impl Sphere {
    pub fn new(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for y in 0..=rings {
            let theta = y as f32 * std::f32::consts::PI / rings as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            
            for x in 0..=segments {
                let phi = x as f32 * 2.0 * std::f32::consts::PI / segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                
                let position = [
                    radius * sin_theta * cos_phi,
                    radius * cos_theta,
                    radius * sin_theta * sin_phi,
                ];
                
                let normal = [
                    sin_theta * cos_phi,
                    cos_theta,
                    sin_theta * sin_phi,
                ];
                
                let tex_coords = [
                    x as f32 / segments as f32,
                    y as f32 / rings as f32,
                ];
                
                vertices.push(SimpleVertex::new(
                    position,
                    tex_coords,
                    normal,
                ));
            }
        }
        
        for y in 0..rings {
            for x in 0..segments {
                let i0 = y * (segments + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + (segments + 1);
                let i3 = i2 + 1;
                
                indices.push(i0);
                indices.push(i2);
                indices.push(i1);
                
                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }
        
        Sphere {
            vertices,
            indices,
        }
    }

    pub fn to_sky_mesh(
        &self, 
        device: &wgpu::Device, 
        sky_pipeline: &SkyPipeline,
        position: [f32; 3],
        sky_uniform: SkyUniform,
    ) -> CustomMesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation(position.into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let sky_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Buffer"),
            contents: bytemuck::cast_slice(&[sky_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: sky_buffer.as_entire_binding(),
            },
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &sky_pipeline.bind_group_layouts.mesh,
            entries: &entries,
            label: None,
        });

        CustomMesh {
            name: "sky".to_string(),
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            bind_group
        }
    }
}

impl ToMesh for Sphere {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::identity()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Mesh {
            name,
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            material: 0
        }
    }
}
