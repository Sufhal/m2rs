use serde::{Deserialize, Serialize};
use crate::modules::assets::assets::load_string;

#[derive(Serialize, Deserialize)]
pub struct MotionsGroup {
    pub name: String,
    pub motions: Vec<Motion>
}

#[derive(Serialize, Deserialize)]
pub struct Motion {
    pub file: String,
    pub probability: u8
}

impl MotionsGroup {
    pub async fn load(motions_file: &str) -> Result<MotionsGroup, MotionsError> {
        let file_content = load_string(motions_file).await.map_err(|e| MotionsError::LoadString(e.to_string()))?;
        let motions_group = serde_json::from_str::<MotionsGroup>(file_content.as_str()).map_err(|e| MotionsError::ParsingError(e.to_string()))?;
        Ok(motions_group)
    }
}

#[derive(Debug)]
enum MotionsError {
    LoadString(String),
    ParsingError(String)
}