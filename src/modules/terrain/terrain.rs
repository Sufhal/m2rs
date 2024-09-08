use std::{collections::HashSet, rc::Rc};
use crate::modules::{assets::gltf_loader::{load_model_glb, load_model_glb_with_name}, core::{model::CustomMesh, object::Object}, environment::environment::Environment, pipelines::sun_pipeline::SunPipeline, state::State};
use super::{chunk::Chunk, property::Property, setting::Setting, texture_set::TextureSet, water::WaterTexture};

pub struct Terrain {
    #[allow(dead_code)]
    setting: Setting,
    water_texture: WaterTexture,
    pub environment: Environment,
    pub chunks: Vec<Chunk>
}

impl Terrain {

    pub async fn load(name: &str, state: &mut State<'_>) -> anyhow::Result<Self> {
        let path = format!("pack/map/{name}");
        let setting = Setting::read(&path).await?;
        let texture_set = TextureSet::read(&path).await?;
        let water_texture = WaterTexture::load(state).await?;
        let environment = Environment::load(&setting.environment, state).await?;
        dbg!(&environment.fog.uniform());
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
        Ok(Self {
            setting,
            chunks,
            environment,
            water_texture
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
