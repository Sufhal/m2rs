use crate::modules::{assets::assets::load_binary, geometry::plane::Plane};

use super::setting::Setting;

pub struct Chunk {
    
}

impl Chunk {
    /// name is something like "000000", "003004"
    pub async fn new(terrain_path: &str, name: &str, setting: &Setting) -> anyhow::Result<Self> {
        let height = load_binary(&format!("{terrain_path}/{name}/height.raw")).await?;
        let u16_height_raw = height
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<u16>>();
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
        let segments = 128u32;
        let size = segments as f32 * 2.0;
        let mut geometry = Plane::new(size, size, segments, segments);
        geometry.set_vertices_height(vertices_height);
        Ok(Self {

        })
    }

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