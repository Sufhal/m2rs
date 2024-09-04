use std::convert::TryInto;

use crate::modules::assets::assets::load_string;


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AreaData {
    pub objects: Vec<AreaObject>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AreaObject {
    pub id: String,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub offset: f32
}

impl AreaData {

    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let filename = format!("{path}/areadata.json");
        let file = load_string(&filename).await?;
        Ok(serde_json::from_str::<Self>(&file)?)
    }
    
    pub fn from_txt(data: &str) -> Self {
        let mut lines = data.lines().map(str::trim);
        let mut objects = Vec::new();
        lines.next(); // AreaDataFile
        while let Some(line) = lines.next() {
            if line.starts_with("Start Object") {
                let position: [f32; 3] = lines.next().unwrap()
                    .split(' ')
                    .map(|v| v.parse::<f32>().unwrap_or(100.0) / 100.0)
                    .collect::<Vec<f32>>()
                    .try_into()
                    .unwrap();
                let id = lines.next().unwrap()
                    .to_string();
                let rotation: [f32; 3] = lines.next().unwrap()
                    .split('#')
                    .map(|v| v.parse::<f32>().unwrap_or(100.0))
                    .collect::<Vec<f32>>()
                    .try_into()
                    .unwrap();
                let offset = lines.next().unwrap()
                    .parse::<f32>().unwrap();
                objects.push(AreaObject {
                    position,
                    id,
                    rotation: [rotation[0], rotation[2], rotation[1]],
                    offset
                });
            }
        }
        Self { objects }
    }
}
