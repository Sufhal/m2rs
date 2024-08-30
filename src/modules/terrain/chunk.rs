use image::{imageops::blur, GrayImage};
use crate::modules::{assets::assets::load_binary, core::{model::TerrainMesh, texture::Texture}, geometry::{buffer::ToTerrainMesh, plane::Plane}, state::State, utils::structs::Set};
use super::setting::Setting;

pub struct Chunk {
    pub mesh: TerrainMesh
}

impl Chunk {
    pub async fn new(
        terrain_path: &str, 
        x: &u8, 
        y: &u8,
        setting: &Setting,
        textures: &Vec<Texture>,
        state: &State<'_>
    ) -> anyhow::Result<Self> {
        let name = Self::name_from(*x, *y);
        let height = load_binary(&format!("{terrain_path}/{name}/height.raw")).await?;
        let tile_indices = load_binary(&format!("{terrain_path}/{name}/tile.raw")).await?;
        // Each vertex is defined by two bytes
        let u16_height_raw = height
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();
        // Delete first and last row and column of height map
        let vertices_height = u16_height_raw
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                const ORIGINAL_SIZE: f64 = 131.0;
                let line_index = f64::floor(*i as f64 / ORIGINAL_SIZE) as u64;
			    let colmun_index = (*i as f64 % ORIGINAL_SIZE) as u64;
                let ignore = line_index == 0 || line_index == 130 || colmun_index == 0 || colmun_index == 130;
                !ignore
            })
            .map(|(_, v)| *v as f32 * setting.height_scale / 100.0) // divided by 100 because original Metin2 is cm based
            .collect::<Vec<_>>();
        // Delete first and last row and column of tile map
        // Replace its values with actual textures indices passed in the shader
        let mut textures_set = Set::new();
        let tile_indices = tile_indices
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                const ORIGINAL_SIZE: f64 = 258.0;
                let line_index = f64::floor(*i as f64 / ORIGINAL_SIZE) as u64;
			    let colmun_index = (*i as f64 % ORIGINAL_SIZE) as u64;
                let ignore = line_index == 0 || line_index == 257 || colmun_index == 0 || colmun_index == 257;
                !ignore
            })
            .map(|(_, v)| {
                let real_index = (*v) - 1;
                textures_set.insert(real_index);
                textures_set.position(&real_index).unwrap() as u8
            })
            .collect::<Vec<_>>();

        let mut alpha_maps = Vec::new();
        for i in 0..textures_set.len() {
            let index = textures_set.get(i).unwrap();
            let alpha_map = tile_indices
                .iter()
                .fold(Vec::<u8>::new(), |mut acc, v| {
                    if *v == *index {
                        acc.push(u8::MAX);
                    } else {
                        acc.push(u8::MIN);
                    }
                    acc
                });
            let blurred_alpha_map: GrayImage = image::ImageBuffer::from_raw(256, 256, alpha_map).unwrap();
            let blurred_alpha_map = blur(&blurred_alpha_map, 2.0);
            alpha_maps.push(
                Texture::from_raw_bytes(
                    &blurred_alpha_map.to_vec(), 
                    256, 
                    256, 
                    wgpu::TextureFormat::R8Unorm, 
                    256, 
                    state
                )
            );
            // std::fs::write(std::path::PathBuf::from(&format!("trash/{name}_{index}.png")), blurred_alpha_map.to_vec());
        }

        let tile = Texture::from_raw_bytes(
            &tile_indices, 
            256, 
            256, 
            wgpu::TextureFormat::R8Unorm, 
            256, 
            state
        );

        let segments = 128u32;
        let size = segments as f32 * 2.0;
        let mut geometry = Plane::new(size, size, segments, segments);
        geometry.set_vertices_height(vertices_height);
        let mesh = geometry.to_terrain_mesh(
            &state.device, 
            &state.terrain_pipeline, 
            name,
            [
                (*x as f32 * size) + (size / 2.0),
                -300.0,
                (*y as f32 * size) + (size / 2.0)
            ],
            &tile,
            textures,
            &alpha_maps,
            &textures_set
        );
        Ok(Self {
            mesh
        })
    }

    /// Something like "001002", "004005"
    pub fn name_from(x: u8, y: u8) -> String {
        fn transform(value: u8) -> String {
            let mut output = format!("{value}");
            while output.len() < 3 {
                output.insert_str(0, "0");
            }
            output
        }
        transform(x) + &transform(y)
    }

}


