use std::convert::TryInto;
use crate::modules::{assets::assets::load_binary, geometry::plane::Plane};

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

            // positions.extend([x, height, y]);
            // positions.extend([x + PATCH_SIZE, height, y]);
            // positions.extend([x, height, y + PATCH_SIZE]);
            // positions.extend([x + PATCH_SIZE, height, y]);
            // positions.extend([x, height, y + PATCH_SIZE]);
            // positions.extend([x + PATCH_SIZE, height, y + PATCH_SIZE]);

            // uvs.extend([x * uv_factor, y * uv_factor]);
			// uvs.extend([(x + PATCH_SIZE) * uv_factor, y * uv_factor]);
			// uvs.extend([x * uv_factor, (y + PATCH_SIZE) * uv_factor]);
			// uvs.extend([(x + PATCH_SIZE) * uv_factor, y * uv_factor]);
			// uvs.extend([x * uv_factor, (y + PATCH_SIZE) * uv_factor]);
			// uvs.extend([(x + PATCH_SIZE) * uv_factor, (y + PATCH_SIZE) * uv_factor]);

            // Ajout des positions et des uvs pour chaque vertex
            positions.extend([x, height, y]);
            uvs.extend([x * uv_factor, y * uv_factor]);

            positions.extend([x + PATCH_SIZE, height, y]);
            uvs.extend([(x + PATCH_SIZE) * uv_factor, y * uv_factor]);

            positions.extend([x, height, y + PATCH_SIZE]);
            uvs.extend([x * uv_factor, (y + PATCH_SIZE) * uv_factor]);

            positions.extend([x + PATCH_SIZE, height, y + PATCH_SIZE]);
            uvs.extend([(x + PATCH_SIZE) * uv_factor, (y + PATCH_SIZE) * uv_factor]);

            // Ajout des indices pour les deux triangles du carré
            indices.extend([
                vertex_offset, vertex_offset + 1, vertex_offset + 2, // Premier triangle
                vertex_offset + 1, vertex_offset + 3, vertex_offset + 2, // Deuxième triangle
            ]);

            vertex_offset += 4; // Chaque itération ajoute 4 nouveaux sommets
        }

        Plane::from(positions, uvs, indices)

    }
}