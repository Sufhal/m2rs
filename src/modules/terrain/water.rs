use std::convert::TryInto;
use crate::modules::{assets::assets::{load_binary, load_png_bytes, load_texture}, core::texture::Texture, geometry::plane::Plane, state::State, utils::functions::u8_to_string_with_len};

const PATCH_SIZE: f32 = 2.0;

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

            indices.extend([
                vertex_offset, vertex_offset + 1, vertex_offset + 2,
                vertex_offset + 1, vertex_offset + 3, vertex_offset + 2,
            ]);

            vertex_offset += 4;
        }

        Plane::from(positions, uvs, indices)

    }
}

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct WaterUniform {
    factor: f32,
    time: f32
}

pub struct WaterTexture {
    pub textures: [Texture; 2],
    pub uniform: WaterUniform,
    textures_data: Vec<Vec<u8>>,
    current: usize,
}

impl WaterTexture {
    pub async fn load(state: &State<'_>) -> anyhow::Result<Self> {
        let mut textures_data = Vec::new();
        for i in 1..=30 {
            let data = load_png_bytes(&format!("pack/special/water/{}.png", u8_to_string_with_len(i, 2))).await?;
            textures_data.push(data);
        }
        let textures = [
            load_texture("pack/special/water/01.png", &state.device, &state.queue).await?,
            load_texture("pack/special/water/02.png", &state.device, &state.queue).await?,
        ];
        Ok(Self {
            textures,
            textures_data,
            current: 0,
            uniform: WaterUniform {
                factor: 0.0,
                time: 0.0,
            }
        })
    }
    pub fn update(&mut self, elapsed_time: f32, queue: &wgpu::Queue) {
        let texture_index = (elapsed_time * 1000.0 / 70.0) % 30.0;
        let current = f32::floor(texture_index);
        let next = if current as usize == self.textures_data.len() - 1 { 0.0 } else { f32::ceil(texture_index) };
        self.uniform.factor = texture_index - current;
        self.uniform.time = elapsed_time;
        let current = current as usize;
        let next = next as usize;
        if current != self.current {
            // texture update needed
            self.current = current;
            self.textures[0].update(&self.textures_data[current], queue);
            self.textures[1].update(&self.textures_data[next], queue);
        }
        
        // const textureIndex = ((elapsedTimeFromStart * 1000) / 70) % 30;
		// const actual = Math.floor(textureIndex);
		// const next = actual === this._textures.length - 1 ? 0 : Math.ceil(textureIndex);
		// this._uniforms.waterTexture.value = this._textures[actual];
		// this._uniforms.waterTextureNext.value = this._textures[next];
		// this._uniforms.factor.value = textureIndex - actual;
		// this._uniforms.time.value = elapsedTimeFromStart;
    }
} 