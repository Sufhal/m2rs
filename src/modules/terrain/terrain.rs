use crate::modules::{core::{model::{DrawCustomMesh, CustomMesh}, texture::TextureAtlas}, pipelines::common_pipeline::CommonPipeline, state::State};
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

    pub fn get_terrain_meshes(&self) -> Vec<&CustomMesh> {
        self.chunks
            .iter()
            .map(|v| &v.terrain_mesh)
            .collect()
    }

    pub fn get_water_meshes(&self) -> Vec<&CustomMesh> {
        self.chunks
            .iter()
            .map(|v| &v.water_mesh)
            .collect()
    }

}
