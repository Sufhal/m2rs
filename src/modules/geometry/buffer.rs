use std::collections::HashSet;

use crate::modules::{core::{model::{Mesh, TerrainMesh}, texture::Texture}, pipelines::terrain_pipeline::TerrainPipeline, utils::structs::Set};

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
        tile: &Texture,
        textures: &Vec<Texture>,
        textures_set: &Set<u8>
    ) -> TerrainMesh;
}