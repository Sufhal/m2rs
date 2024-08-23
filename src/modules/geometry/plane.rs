
use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::modules::core::model::{Mesh, ModelVertex, TransformUniform};

use super::buffer::ToMesh;

pub struct Plane {
    vertices: Vec<ModelVertex>,
    indices: Vec<u32>,
}

impl Plane {
    pub fn new(width: f32, height: f32, segments_x: u32, segments_y: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let segment_width = width / segments_x as f32;
        let segment_height = height / segments_y as f32;
        let normal = [0.0, 1.0, 0.0]; // Normale pointant vers le haut
        for y in 0..=segments_y {
            for x in 0..=segments_x {
                let position = [
                    x as f32 * segment_width - width / 2.0,
                    0.0,
                    y as f32 * segment_height - height / 2.0,
                ];

                let tex_coords = [
                    x as f32 / segments_x as f32,
                    y as f32 / segments_y as f32,
                ];

                vertices.push(ModelVertex::new(
                    position,
                    tex_coords,
                    normal,
                    [0,0,0,0],
                    [0.0, 0.0, 0.0, 0.0],
                ));
            }
        }
        for y in 0..segments_y {
            for x in 0..segments_x {
                let i0 = y * (segments_x + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + (segments_x + 1);
                let i3 = i2 + 1;

                indices.push(i0);
                indices.push(i2);
                indices.push(i1);

                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }
        Plane {
            vertices,
            indices,
        }
    }
}

impl ToMesh for Plane {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
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