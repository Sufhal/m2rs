use serde::{Deserialize, Serialize};
use crate::modules::{assets::assets::load_string, utils::functions::random_u8};

use super::skinning::{AnimationMixer, MixerState};

#[derive(Serialize, Deserialize)]
pub struct MotionsGroups {
    pub groups: Vec<MotionsGroup>,
}

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

impl MotionsGroups {
    pub async fn load(motions_file: &str) -> Result<Self, MotionsError> {
        let file_content = load_string(motions_file).await.map_err(|e| MotionsError::LoadString(format!("Unable to load file {motions_file} -> {}", e.to_string())))?;
        let motions_group = serde_json::from_str::<MotionsGroups>(file_content.as_str()).map_err(|e| MotionsError::ParsingError(e.to_string()))?;
        Ok(motions_group)
    }
    pub fn update_mixer(&self, mixer: &mut AnimationMixer) {
        dbg!(&mixer.state);
        if let MixerState::None = mixer.state {
            // println!("picking motion");
            let motion = self.groups[0].pick_motion();
            mixer.play(&motion.file);
        }
    }
}

impl MotionsGroup {
    fn pick_motion(&self) -> &Motion {
        let mut cumulative_probability = 0u8;
        let random = random_u8(100);
        dbg!(&random);
        for motion in &self.motions {
            cumulative_probability += motion.probability;
            if random < cumulative_probability {
                println!("selected motion is {}", motion.file);
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