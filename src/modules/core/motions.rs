use serde::{Deserialize, Serialize};
use crate::modules::{assets::assets::load_string, utils::functions::random_u8};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionsGroups {
    pub groups: Vec<MotionsGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionsGroup {
    pub name: String,
    pub motions: Vec<Motion>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Motion {
    pub file: String,
    pub probability: u8
}

impl MotionsGroups {
    pub async fn load(motions_file: &str) -> Result<Self, MotionsError> {
        let file_content = load_string(motions_file).await.map_err(|e| MotionsError::LoadString(format!("Unable to load file {motions_file} -> {}", e.to_string())))?;
        let motions_group = serde_json::from_str::<MotionsGroups>(file_content.as_str()).map_err(|e| MotionsError::ParsingError(e.to_string()))?;
        Ok(motions_group)
    }
    pub fn get_group(&self, motion_name: &str) -> Option<&MotionsGroup> {
        self.groups.iter().find(|group| group.name == motion_name)
    }
}

impl MotionsGroup {
    pub fn pick_motion(&self) -> &Motion {
        let mut cumulative_probability = 0u8;
        let random = random_u8(100);
        for motion in &self.motions {
            cumulative_probability += motion.probability;
            if random < cumulative_probability {
                return motion;
            }
        }
        // if cumulated probabilities is < 100, it should not happend mais vraiment la flemme de gérer une erreur pour ça
        self.motions.get(0).as_ref().unwrap()
    }
}

#[derive(Debug)]
pub enum MotionsError {
    LoadString(String),
    ParsingError(String)
}