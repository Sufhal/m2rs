use wgpu::util::DeviceExt;

use crate::modules::{assets::{assets::load_binary, gltf_loader::load_model_glb}, core::{model::CustomMesh, object_3d::Object3D, texture::Texture}, geometry::plane::Plane, state::State, utils::functions::u8_to_string_with_len};
use super::{areadata::AreaData, height::Height, setting::Setting, texture_set::ChunkTextureSet, water::{Water, WaterTexture}};

pub struct Chunk {
    pub terrain_mesh: CustomMesh,
    pub water_mesh: CustomMesh,
    pub depth_buffer: wgpu::Buffer,
    water_buffer: wgpu::Buffer,
}

impl Chunk {
    pub async fn new(
        terrain_path: &str, 
        x: &u8, 
        y: &u8,
        setting: &Setting,
        textures: &Vec<Texture>,
        water_textures: &WaterTexture,
        state: &mut State<'_>
    ) -> anyhow::Result<Self> {
        let name = Self::name_from(*x, *y);
        let chunk_path = &format!("{terrain_path}/{name}");
        let height = Height::read(&chunk_path, setting.height_scale).await?;
        let water = Water::read(&chunk_path).await?;
        // dbg!(&water);
        let textures_set = ChunkTextureSet::read(&chunk_path).await?;
        let mut alpha_maps = Vec::new();
        for i in 0..textures_set.textures.len() {
            let alpha_map_raw = load_binary(&format!("{chunk_path}/tile_{i}.raw")).await?;
            alpha_maps.push(
                Texture::from_raw_bytes(
                    &alpha_map_raw, 
                    256, 
                    256, 
                    wgpu::TextureFormat::R8Unorm, 
                    wgpu::FilterMode::Nearest,
                    256, 
                    state
                )
            );
        }
        let segments = 128u32;
        let size = segments as f32 * 2.0;
        let position = [
            (*x as f32 * size) + (size / 2.0),
            0.0,
            (*y as f32 * size) + (size / 2.0)
        ];
        let mut terrain_geometry = Plane::new(size, size, segments, segments);
        terrain_geometry.set_vertices_height(height.vertices);
        let terrain_mesh = terrain_geometry.to_terrain_mesh(
            &state.device, 
            &state.terrain_pipeline, 
            name.clone(),
            position.clone(),
            textures,
            &alpha_maps,
            &textures_set
        );
        let water_geometry = water.generate_plane(setting.height_scale);
        let depth = Water::calculate_depth(&water_geometry, &terrain_geometry);
        let depth_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Depth Buffer"),
            contents: bytemuck::cast_slice(&depth),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let (water_mesh, water_buffer) = water_geometry.to_water_mesh(
            &state.device, 
            &state.water_pipeline, 
            name,
            [
                (*x as f32 * size),
                0.0,
                (*y as f32 * size)
            ],
            &water_textures,
        );
        let areadata = AreaData::read(&chunk_path).await?;
        let model_objects = load_model_glb(
            "pack/zone/c/building/c1-001-house3.glb", 
            &state.device, 
            &state.queue, 
            &state.skinned_models_pipeline,
            &state.simple_models_pipeline,
        ).await.expect("unable to load");
        for mut object in model_objects {
            if let Some(object3d) = &mut object.object3d {
                match object3d {
                    Object3D::Simple(simple) => {
                        for object in &areadata.objects {
                            // println!("{:?}", object.position);
                            let instance = simple.request_instance(&state.device);
                            instance.set_position(cgmath::Vector3::from([
                                object.position[0],
                                object.position[1],
                                object.position[2]
                            ]));
                            instance.take();
                        }
                    },
                    _ => ()
                };
            }
            state.scene.add(object);
        }
        
        Ok(Self {
            terrain_mesh,
            water_mesh,
            water_buffer,
            depth_buffer,
        })
    }

    /// Something like "001002", "004005"
    pub fn name_from(x: u8, y: u8) -> String {
        u8_to_string_with_len(x, 3) + &u8_to_string_with_len(y, 3)
    }

    pub fn update(&self, water_texture: &WaterTexture, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.water_buffer,
            0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&[water_texture.uniform]),
        );
    }


}


#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct ChunkInformationUniform {
    pub textures_count: u32
}