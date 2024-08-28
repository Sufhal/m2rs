use cgmath::SquareMatrix;

use crate::modules::{core::texture, pipelines::{common_pipeline::{self, CommonPipeline}, render_pipeline::RenderPipeline}};
use std::ops::Range;

use super::skinning::{AnimationClip, Skeleton};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedMeshVertex {
    pub position: [f32; 3],     // 12 octets
    pub tex_coords: [f32; 2],   // 8 octets
    pub normal: [f32; 3],       // 12 octets
    pub weights: [f32; 4],      // 16 octets
    pub joints: [u32; 4],       // 16 octets
}

impl SkinnedMeshVertex {
    pub fn new(
        position: [f32; 3], 
        tex_coords: [f32; 2], 
        normal: [f32; 3], 
        joints: [u32; 4], 
        weights: [f32; 4]
    ) -> Self {
        SkinnedMeshVertex {
            position,
            tex_coords,
            normal,
            weights,
            joints,
        }
    }
}

impl Vertex for SkinnedMeshVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SkinnedMeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],     
    pub tex_coords: [f32; 2],  
    pub normal: [f32; 3],
    // pub texture_indices: [f32; 4],
}

impl TerrainVertex {
    pub fn new(
        position: [f32; 3], 
        tex_coords: [f32; 2], 
        normal: [f32; 3], 
        // texture_indices: [f32; 4]
    ) -> Self {
        TerrainVertex {
            position,
            tex_coords,
            normal,
            // texture_indices
        }
    }
}

impl Vertex for TerrainVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // wgpu::VertexAttribute {
                //     offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                //     shader_location: 3,
                //     format: wgpu::VertexFormat::Float32x4,
                // }
            ],
        }
    }
}

#[derive(Debug)]
pub struct SkinnedModel {
    pub meshes: Vec<Mesh>,
    pub skeleton: Skeleton,
    pub animations: Vec<AnimationClip>,
    pub materials: Vec<Material>,
    pub meshes_bind_groups: Vec<wgpu::BindGroup>,
}

impl SkinnedModel {
    /// Creates one BindGroup per Mesh
    pub fn create_bind_groups(&mut self, device: &wgpu::Device, render_pipeline: &RenderPipeline) {
        self.meshes_bind_groups.clear();
        for mesh in &self.meshes {
            self.meshes_bind_groups.push(
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &render_pipeline.bind_group_layouts.mesh,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.materials[mesh.material].diffuse_texture.view
                            )
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource:  wgpu::BindingResource::Sampler(
                                &self.materials[mesh.material].diffuse_texture.sampler
                            )
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: mesh.transform_buffer.as_entire_binding(),
                        }
                    ],
                    label: None,
                })
            )
        }
        
    }
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: texture::Texture
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    transform: [[f32; 4]; 4],
}

impl TransformUniform {
    pub fn from(raw_matrix: [[f32; 4]; 4]) -> Self {
        TransformUniform {
            transform: raw_matrix
        }
    }
    pub fn identity() -> Self {
        TransformUniform {
            transform: cgmath::Matrix4::identity().into()
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub transform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub struct TerrainMesh {
    pub name: String,
    pub transform_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub bind_group: wgpu::BindGroup,
}


pub trait DrawMesh<'a> {
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        mesh_bind_group: &'a wgpu::BindGroup,
        instances_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawMesh<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        mesh_bind_group: &'b wgpu::BindGroup,
        instances_bind_group: &'b wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &common_pipeline.global_bind_group, &[]);
        self.set_bind_group(1, &mesh_bind_group, &[]);
        self.set_bind_group(2, &instances_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}


pub trait DrawSkinnedModel<'a> {
    fn draw_model_instanced(
        &mut self,
        model: &'a SkinnedModel,
        instances_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawSkinnedModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_model_instanced(
        &mut self,
        model: &'b SkinnedModel,
        instances_bind_group: &'b wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    ) {
        for i in 0..model.meshes.len() {
            let mesh = &model.meshes[i];
            let mesh_bind_group = &model.meshes_bind_groups[i];
            self.draw_mesh_instanced(mesh, mesh_bind_group, instances_bind_group, instances.clone(), render_pipeline, common_pipeline);
        }
    }
}


pub trait DrawTerrainMesh<'a> {
    fn draw_terrain_mesh(
        &mut self,
        terrain_mesh: &'a TerrainMesh,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawTerrainMesh<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_terrain_mesh(
        &mut self,
        terrain_mesh: &'a TerrainMesh,
        common_pipeline: &'a CommonPipeline,
    ) {
        self.set_vertex_buffer(0, terrain_mesh.vertex_buffer.slice(..));
        self.set_index_buffer(terrain_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &common_pipeline.global_bind_group, &[]);
        self.set_bind_group(1, &terrain_mesh.bind_group, &[]);
        self.draw_indexed(0..terrain_mesh.num_elements, 0, 0..1);
    }
}
