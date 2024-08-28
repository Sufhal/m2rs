use crate::modules::{core::{model::{Mesh, TerrainMesh}, texture::Texture}, pipelines::terrain_pipeline::TerrainPipeline};

pub trait ToMesh {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh;
}

pub trait ToTerrainMesh {
    fn to_terrain_mesh(
        &self, 
        device: &wgpu::Device, 
        terrain_pipeline: &TerrainPipeline, 
        name: String, 
        position: [f32; 3],
        texture: &Texture
    ) -> TerrainMesh;
}