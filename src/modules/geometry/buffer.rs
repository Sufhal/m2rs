use wgpu::util::DeviceExt;
use crate::modules::core::model::{Mesh, ModelVertex};

pub trait ToMesh {
    fn to_mesh(device: wgpu::Device, name: String, vertices: Vec<f32>, indices: Vec<u32>) -> Mesh {
        let vertices = (0..vertices.len() / 3)
            .map(|i| {
                ModelVertex {
                    position: [
                        vertices[i * 3],
                        vertices[i * 3 + 1],
                        vertices[i * 3 + 2],
                    ],
                    tex_coords: [0.0, 0.0],
                    normal: [0.0, 0.0, 0.0],
                }
            })
            .collect::<Vec<_>>();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Mesh {
            name,
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0
        }
    }
}