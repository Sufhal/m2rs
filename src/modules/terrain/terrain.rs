use std::collections::HashSet;

use crate::modules::{assets::gltf_loader::load_model_glb, core::model::CustomMesh, state::State};
use super::{chunk::Chunk, property::Property, setting::Setting, texture_set::TextureSet, water::WaterTexture};

pub struct Terrain {
    #[allow(dead_code)]
    setting: Setting,
    water_texture: WaterTexture,
    pub chunks: Vec<Chunk>
}

impl Terrain {

    pub async fn load(name: &str, state: &mut State<'_>) -> anyhow::Result<Self> {
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
        let mut properties = HashSet::new();
        for chunk in &mut chunks {
            for property in chunk.get_properties_to_preload() {
                properties.insert(property);
            }
        }
        // load all properties

        // let mut futures = Vec::new();
        // for property_id in &properties {
        //     if let Some(property) = state.properties.properties.get(property_id) {
        //         match property {
        //             Property::Building(building) => {
        //                 let future = load_model_glb(
        //                     &building.file, 
        //                     &state.device, 
        //                     &state.queue, 
        //                     &state.skinned_models_pipeline,
        //                     &state.simple_models_pipeline,
        //                 );
        //                 futures.push(future);
        //             }
        //         }
        //     }
        // }
        
        // load all objects
        for chunk in &mut chunks {
            chunk.load_objects_instances(state).await;
        }
        Ok(Self {
            setting,
            chunks,
            water_texture
        })
    }

    pub fn update(&mut self, elapsed_time: f32, queue: &wgpu::Queue) {
        self.water_texture.update(elapsed_time);
        for chunk in &self.chunks {
            chunk.update(&self.water_texture, queue);
        }
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
