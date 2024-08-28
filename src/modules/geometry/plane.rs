
use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;
use crate::modules::{core::{model::{Mesh, TerrainMesh, TerrainVertex, TransformUniform}, texture::Texture}, pipelines::terrain_pipeline::TerrainPipeline};
use super::buffer::{ToMesh, ToTerrainMesh};

pub struct Plane {
    vertices: Vec<TerrainVertex>,
    indices: Vec<u32>,
}

impl Plane {
    pub fn new(width: f32, height: f32, segments_x: u32, segments_y: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let segment_width = width / segments_x as f32;
        let segment_height = height / segments_y as f32;
        let normal = [0.0, 1.0, 0.0]; // Normale pointant vers le haut

        let mut offset = 0usize;
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

                // let texture_indices = [
                //     textures_indices[offset + 0],
                //     textures_indices[offset + 1],
                //     textures_indices[offset + 2],
                //     textures_indices[offset + 3],
                // ];

                vertices.push(TerrainVertex::new(
                    position,
                    tex_coords,
                    normal,
                    // texture_indices
                ));
                offset += 4;
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
        println!("indices {}", indices.len());
        Plane {
            vertices,
            indices,
        }
    }

    pub fn set_vertices_height(&mut self, vertices_height: Vec<f32>) {
        if vertices_height.len() != self.vertices.len() {
            panic!("Impossible to set vertices height with incompatible surface vertices count");
        }
        for i in 0..vertices_height.len() {
            self.vertices[i].position[1] = vertices_height[i];
        }
    }

}

impl ToTerrainMesh for Plane {
    fn to_terrain_mesh(
        &self, 
        device: &wgpu::Device, 
        terrain_pipeline: &TerrainPipeline, 
        name: String, 
        position: [f32; 3], 
        texture: &Texture
    ) -> TerrainMesh {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Terrain Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation(position.into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &terrain_pipeline.bind_group_layouts.mesh,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource:  wgpu::BindingResource::Sampler(&texture.sampler)
                },
            ],
            label: None,
        });
        TerrainMesh {
            name,
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: self.indices.len() as u32,
            bind_group
        }
    }
}