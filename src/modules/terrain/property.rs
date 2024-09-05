use std::{collections::HashMap, convert::TryInto};

use crate::modules::assets::assets::load_string;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Properties {
    pub properties: Vec<Property>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Property {
    Building(Building)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Building {
    pub id: String,
    pub file: String,
    pub name: String,
    pub shadow_flag: String,
}

impl Properties {

    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let filename = format!("{path}/properties.json");
        let file = load_string(&filename).await?;
        Ok(serde_json::from_str::<Self>(&file)?)
    }

}

impl Property {
    
    pub fn from_txt(data: &str) -> Option<Self> {
        let mut lines = data.lines().map(str::trim);
        lines.next(); // YPRT
        let id = lines.next().unwrap().to_string();
        let mut keyval = HashMap::new();
        while let Some(line) = lines.next() {
            let [key, val]: [&str; 2] = line
                .split_whitespace()
                .map(str::trim)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            keyval.insert(key, val);
        }
        match keyval.get("propertytype") {
            Some(&"Building") => {
                Some(Self::Building(Building {
                    id,
                    file: keyval.get("buildingfile").unwrap().to_string(),
                    name: keyval.get("propertyname").unwrap().to_string(),
                    shadow_flag: keyval.get("shadowflag").unwrap().to_string(),
                }))
            },
            _ => None
        }
    }
}
