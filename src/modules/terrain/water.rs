use std::{collections::{BTreeMap, HashMap}, convert::TryInto};
use rustc_hash::FxHashMap;

use crate::modules::{assets::assets::{load_binary, load_png_bytes, load_texture}, core::{model::SimpleVertex, texture::Texture}, geometry::plane::Plane, state::State, utils::functions::u8_to_string_with_len};

const PATCH_SIZE: f32 = 2.0;
const TEXTURES_COUNT: usize = 30;

#[derive(Debug)]
pub struct Water {
    pub size: [u8; 2],
    pub layers_count: u8,
    pub layers_per_vertex: Vec<u8>,
    pub layers_height: Vec<u32>
}

impl Water {
    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let raw = load_binary(&format!("{path}/water.wtr")).await?;
        const HEADER_START: usize = 0;
		const HEADER_STOP: usize = 7;
        let header = &raw[HEADER_START..HEADER_STOP];
        let size = [header[2], header[4]];
        let layers_count = header[6];
        let content_stop = HEADER_STOP + (size[0] as usize * size[1] as usize);
        let layers_per_vertex = raw[HEADER_STOP..content_stop].to_vec();
        let layers_height_stop = content_stop + (layers_count as usize * 4);
        let footer = &raw[content_stop..layers_height_stop];
        let layers_height = footer
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
            .collect();
        Ok(Self {
            size,
            layers_count,
            layers_per_vertex,
            layers_height
        })
    }
    
    pub fn generate_plane(&self, height_scale: f32) -> Plane {
        let water_height = self.layers_per_vertex
            .iter()
            .map(|v| if *v == u8::MAX { 0u32 } else { self.layers_height[*v as usize] })
            .collect::<Vec<_>>();

        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut indices_positions = FxHashMap::default();

        let size = f32::sqrt(water_height.len() as f32);
        let uv_factor = 1.0 / size;
        let mut vertex_offset = 0;

        for i in 0..water_height.len() {
            let y = f32::floor(i as f32 / size) * PATCH_SIZE;
            let x = (i as f32 % size) * PATCH_SIZE;
            let height = water_height[i] as f32 * height_scale;

            positions.extend([x, height, y]);
            uvs.extend([x * uv_factor, y * uv_factor]);

            positions.extend([x + PATCH_SIZE, height, y]);
            uvs.extend([(x + PATCH_SIZE) * uv_factor, y * uv_factor]);

            positions.extend([x, height, y + PATCH_SIZE]);
            uvs.extend([x * uv_factor, (y + PATCH_SIZE) * uv_factor]);

            positions.extend([x + PATCH_SIZE, height, y + PATCH_SIZE]);
            uvs.extend([(x + PATCH_SIZE) * uv_factor, (y + PATCH_SIZE) * uv_factor]);

            let vertex_index = vertex_offset as usize;

            indices_positions.insert(vertex_index + 0, indices.len() + 0);
            indices_positions.insert(vertex_index + 2, indices.len() + 1);
            indices_positions.insert(vertex_index + 1, indices.len() + 2);
            indices_positions.insert(vertex_index + 3, indices.len() + 5);

            indices.extend([
                vertex_offset, 
                vertex_offset + 2, 
                vertex_offset + 1,
                vertex_offset + 1, 
                vertex_offset + 2,
                vertex_offset + 3,
            ]);

            vertex_offset += 4;
        }
        Plane::from(positions, uvs, indices, indices_positions)
    }

    pub fn calculate_depth(
        water_plane: &Plane,
        terrain_plane: &Plane,
    ) -> Vec<f32> {
        // dbg!(water_plane.indices.len(), water_plane.vertices.len(), terrain_plane.indices.len(), terrain_plane.vertices.len());
        let mut depth: Vec<f32> = Vec::new();
        for i in 0..water_plane.vertices.len() {
            let indice_position = water_plane.indices_positions.get(&i).unwrap();
            let water_height = water_plane.vertices[i].position[1];
            let terrain_height = terrain_plane.vertices[terrain_plane.indices[*indice_position] as usize].position[1];
            depth.push(water_height - terrain_height);
            // depth.push(1.0);
        }
        depth
    }

}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct WaterUniform {
    factor: f32,
    time: f32,
    current: u32,
    count: u32,
}

pub struct WaterTexture {
    pub atlas_texture: Texture,
    pub uniform: WaterUniform,
}

impl WaterTexture {
    pub async fn load(state: &State<'_>) -> anyhow::Result<Self> {
        let atlas_texture = load_texture("pack/special/water/atlas.png", &state.device, &state.queue).await?;
        Ok(Self {
            atlas_texture,
            uniform: WaterUniform {
                factor: 0.0,
                time: 0.0,
                current: 0,
                count: TEXTURES_COUNT as u32
            }
        })
    }
    pub fn update(&mut self, elapsed_time: f32) {
        let texture_index = (elapsed_time * 1000.0 / 70.0) % 30.0;
        let current = f32::floor(texture_index);
        let next = if current as usize == TEXTURES_COUNT - 1 { 0.0 } else { f32::ceil(texture_index) };
        self.uniform.factor = texture_index - current;
        self.uniform.time = elapsed_time;
    }
} 

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WaterDepth {
    depth: f32,
}

impl WaterDepth {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<WaterDepth>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
