use crate::modules::assets::assets::load_string;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Setting {
    pub cell_scale: u64,
    pub height_scale: f32,
    pub view_radius: u64,
    pub map_size: [u8; 2],
    pub base_position: [u64; 2],
    pub texture_set: String,
    pub environment: String
}

impl Setting {
    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let filename = format!("{path}/setting.json");
        let file = load_string(&filename).await?;
        let setting = serde_json::from_str::<Setting>(&file)?;
        Ok(setting)
    }
}