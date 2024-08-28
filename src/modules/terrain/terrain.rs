use crate::modules::{core::model::{DrawTerrainMesh, TerrainMesh}, pipelines::common_pipeline::CommonPipeline, state::State};
use super::{chunk::Chunk, setting::Setting};

pub struct Terrain {
    setting: Setting,
    chunks: Vec<Chunk>
}

impl Terrain {

    pub async fn load(name: &str, state: &State<'_>) -> anyhow::Result<Self> {
        let path = format!("pack/map/{name}");
        let setting = Setting::read(&path).await?;
        let mut chunks = Vec::new();
        for x in 0..setting.map_size[0] {
            for y in 0..setting.map_size[1] {
                let chunk = Chunk::new(
                    &path,
                    &x, 
                    &y,
                    &setting,
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

