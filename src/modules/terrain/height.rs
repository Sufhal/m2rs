use crate::modules::assets::assets::load_binary;

const ORIGINAL_SIZE: f64 = 131.0;

pub struct Height {
    pub vertices: Vec<f32>
}

impl Height {
    pub async fn read(path: &str, height_scale: f32) -> anyhow::Result<Self> {
        let height = load_binary(&format!("{path}/height.raw")).await?;
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
                let line_index = f64::floor(*i as f64 / ORIGINAL_SIZE) as u64;
			    let colmun_index = (*i as f64 % ORIGINAL_SIZE) as u64;
                let ignore = line_index == 0 || line_index == 130 || colmun_index == 0 || colmun_index == 130;
                !ignore
            })
            .map(|(_, v)| *v as f32 * height_scale) 
            .collect::<Vec<_>>();
        Ok(Self {
            vertices: vertices_height
        })
    }
}