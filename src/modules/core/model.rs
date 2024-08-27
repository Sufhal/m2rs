use cgmath::SquareMatrix;

use crate::modules::{core::texture, pipelines::{common_pipeline::{self, CommonPipeline}, render_pipeline::RenderPipeline}};
use std::ops::Range;

use super::skinning::{AnimationClip, Skeleton};

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkinnedVertex {
    pub position: [f32; 3],     // 12 octets
    pub tex_coords: [f32; 2],   // 8 octets
    pub normal: [f32; 3],       // 12 octets
    pub weights: [f32; 4],      // 16 octets
    pub joints: [u32; 4],       // 16 octets
}

impl SkinnedVertex {
    pub fn new(
        position: [f32; 3], 
        tex_coords: [f32; 2], 
        normal: [f32; 3], 
        joints: [u32; 4], 
        weights: [f32; 4]
    ) -> Self {
        SkinnedVertex {
            position,
            tex_coords,
            normal,
            weights,
            joints,
        }
    }
}

impl Vertex for SkinnedVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SkinnedVertex>() as wgpu::BufferAddress,
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
pub struct MeshVertex {
    pub position: [f32; 3],     
    pub tex_coords: [f32; 2],  
    pub normal: [f32; 3],
}

impl MeshVertex {
    pub fn new(
        position: [f32; 3], 
        tex_coords: [f32; 2], 
        normal: [f32; 3], 
    ) -> Self {
        MeshVertex {
            position,
            tex_coords,
            normal,
        }
    }
}

impl Vertex for MeshVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
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
                }
            ],
        }
    }
}

#[derive(Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub skeleton: Skeleton,
    pub animations: Vec<AnimationClip>,
    pub materials: Vec<Material>,
    pub meshes_bind_groups: Vec<wgpu::BindGroup>,
}

impl Model {
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
 
// model.rs
pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        mesh_bind_group: &'a wgpu::BindGroup,
        instances_bind_group: &'a wgpu::BindGroup,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        mesh_bind_group: &'a wgpu::BindGroup,
        instances_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );

    fn draw_model(
        &mut self,
        model: &'a Model,
        instances_bind_group: &'a wgpu::BindGroup,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances_bind_group: &'a wgpu::BindGroup,
        instances: Range<u32>,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        mesh_bind_group: &'b wgpu::BindGroup,
        instances_bind_group: &'b wgpu::BindGroup,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    ) {
        self.draw_mesh_instanced(mesh, mesh_bind_group, instances_bind_group, 0..1, render_pipeline, common_pipeline);
    }

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
        // self.set_bind_group(3, &mesh.transform_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        instances_bind_group: &'b wgpu::BindGroup,
        render_pipeline: &'a RenderPipeline,
        common_pipeline: &'a CommonPipeline,
    ) {
        self.draw_model_instanced(model, instances_bind_group, 0..1, render_pipeline, common_pipeline);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
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

// model.rs
pub trait DrawLight<'a> {
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'b Mesh,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }
    fn draw_light_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(mesh, instances.clone(), camera_bind_group, light_bind_group);
        }
    }
}
