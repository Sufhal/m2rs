use crate::modules::{core::{model::{DrawTerrainMesh, TerrainMesh}, texture::TextureAtlas}, pipelines::common_pipeline::CommonPipeline, state::State};
use super::{chunk::Chunk, setting::Setting, texture_set::TextureSet};

pub struct Terrain {
    setting: Setting,
    chunks: Vec<Chunk>
}

impl Terrain {

    pub async fn load(name: &str, state: &State<'_>) -> anyhow::Result<Self> {
        let path = format!("pack/map/{name}");
        let setting = Setting::read(&path).await?;
        let texture_set = TextureSet::read(&path).await?;
        let textures = texture_set.load_textures(&state.device, &state.queue).await?;
        let mut chunks = Vec::new();
        for x in 0..setting.map_size[0] {
            for y in 0..setting.map_size[1] {
                let chunk = Chunk::new(
                    &path,
                    &x, 
                    &y,
                    &setting,
                    &textures,
                    state
                ).await?;
                chunks.push(chunk);
            }
        }
        Ok(Self {
            setting,
            chunks
        })
    }

    pub fn get_meshes(&self) -> Vec<&TerrainMesh> {
        self.chunks
            .iter()
            .map(|chunk| &chunk.mesh)
            .collect()
    }

}

pub trait DrawTerrain<'a> {
    fn draw_terrain(
        &mut self,
        queue: &wgpu::Queue,
        terrain: &'a Terrain,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawTerrain<'b> for wgpu::RenderPass<'a>
where 
    'b: 'a,
{
    fn draw_terrain(
        &mut self,
        queue: &wgpu::Queue,
        terrain: &'b Terrain,
        common_pipeline: &'a CommonPipeline,
    ) {
        for chunk in terrain.get_meshes() {
            self.draw_terrain_mesh(chunk, common_pipeline);
        }
    }
}

