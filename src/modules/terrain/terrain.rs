use std::collections::HashSet;
use crate::modules::{assets::gltf_loader::load_model_glb_with_name, core::model::CustomMesh, environment::environment::Environment, state::State};
use super::{chunk::Chunk, property::Property, setting::Setting, texture_set::TextureSet, water::WaterTexture};

pub struct Terrain {
    water_texture: WaterTexture,
    pub setting: Setting,
    pub environment: Environment,
    pub chunks: Vec<Chunk>,
    pub center: [f32; 3],
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
        let mut futures = Vec::new();
        for property_id in &properties {
            if let Some(property) = state.properties.properties.get(property_id) {
                match property {
                    Property::Building(building) => {
                        let future = load_model_glb_with_name(
                            &building.file, 
                            &property_id,
                            &state.device, 
                            &state.queue, 
                            &state.skinned_models_pipeline,
                            &state.simple_models_pipeline,
                        );
                        futures.push(future);
                    }
                }
            }
        }
        for result in futures::future::join_all(futures).await {
            if let Ok(objects) = result {
                for object in objects {
                    state.scene.add(object);
                }
            }
        }
        // load all objects
        for chunk in &mut chunks {
            chunk.load_objects_instances(state).await;
        }
        let center = {
            let y = chunks.iter().fold(0.0, |acc, v| acc + v.mean_height) / chunks.len() as f32;
            let x = (setting.map_size[0] as f32 * 256.0) / 2.0;
            let z = (setting.map_size[1] as f32 * 256.0) / 2.0;
            [x, y, z]
        };
        let environment = Environment::load(&setting.environment, center, state).await?;
        Ok(Self {
            setting,
            chunks,
            environment,
            water_texture,
            center,
        })
    }

    pub fn update(&mut self, elapsed_time: f32, delta: f32, queue: &wgpu::Queue) {
        self.water_texture.update(elapsed_time);
        self.environment.update(delta, queue);
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
