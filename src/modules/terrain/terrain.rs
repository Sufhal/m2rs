use crate::modules::{core::{model::{DrawCustomMesh, CustomMesh}, texture::TextureAtlas}, pipelines::common_pipeline::CommonPipeline, state::State};
use super::{chunk::Chunk, setting::Setting, texture_set::TextureSet, water::WaterTexture};

pub struct Terrain {
    setting: Setting,
    water_texture: WaterTexture,
    chunks: Vec<Chunk>
}

impl Terrain {

    pub async fn load(name: &str, state: &State<'_>) -> anyhow::Result<Self> {
        let path = format!("pack/map/{name}");
        let setting = Setting::read(&path).await?;
        let texture_set = TextureSet::read(&path).await?;
        let water_texture = WaterTexture::load(state).await?;
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
                    &water_texture,
                    state
                ).await?;
                chunks.push(chunk);
            }
        }
        Ok(Self {
            setting,
            chunks,
            water_texture
        })
    }

    pub fn update(&mut self, elapsed_time: f32, queue: &wgpu::Queue) {
        self.water_texture.update(elapsed_time, queue);
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
