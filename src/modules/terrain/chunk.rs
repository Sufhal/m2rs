use crate::modules::{assets::assets::load_binary, core::{model::CustomMesh, texture::Texture}, geometry::plane::Plane, state::State, utils::functions::u8_to_string_with_len};
use super::{height::Height, setting::Setting, texture_set::ChunkTextureSet, water::{Water, WaterTexture}};

pub struct Chunk {
    pub terrain_mesh: CustomMesh,
    pub water_mesh: CustomMesh,
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
        state: &State<'_>
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
                    256, 
                    state
                )
            );
        }
        let segments = 128u32;
        let size = segments as f32 * 2.0;
        let mut terrain_geometry = Plane::new(size, size, segments, segments);
        terrain_geometry.set_vertices_height(height.vertices);
        let terrain_mesh = terrain_geometry.to_terrain_mesh(
            &state.device, 
            &state.terrain_pipeline, 
            name.clone(),
            [
                (*x as f32 * size) + (size / 2.0),
                -300.0,
                (*y as f32 * size) + (size / 2.0)
            ],
            textures,
            &alpha_maps,
            &textures_set
        );
        let water_geometry = water.generate_plane(setting.height_scale);
        let (water_mesh, water_buffer) = water_geometry.to_water_mesh(
            &state.device, 
            &state.water_pipeline, 
            name,
            [
                (*x as f32 * size),
                -300.0,
                (*y as f32 * size)
            ],
            [
                &water_textures.textures[0],
                &water_textures.textures[1],
            ],
            water_textures.uniform.clone()
        );
        Ok(Self {
            terrain_mesh,
            water_mesh,
            water_buffer
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