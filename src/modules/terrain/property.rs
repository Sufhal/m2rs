use std::collections::HashMap;
use crate::modules::{assets::assets::load_string, conversion::common::bye_ymir};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Properties {
    pub properties: HashMap<String, Property>
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

    pub async fn read() -> anyhow::Result<Self> {
        let filename = format!("pack/property/properties.json");
        let file = load_string(&filename).await?;
        Ok(serde_json::from_str::<Self>(&file)?)
    }

}

impl Property {
    
    pub fn from_txt(data: &str) -> Option<(String, Self)> {
        let mut lines = data.lines().map(str::trim);
        lines.next()?; // YPRT
        let id = lines.next()?.to_string();
        let mut keyval = HashMap::new();
        while let Some(line) = lines.next() {
            let values = line
                .split("		")
                .map(|v| v.trim_matches('"'))
                .collect::<Vec<_>>();
            if values.len() != 2 {
                return None
            }
            keyval.insert(values[0], values[1]);
        }
        match keyval.get("propertytype") {
            Some(&"Building") => {
                Some((
                    id.clone(),
                    Self::Building(Building {
                        id,
                        file: bye_ymir(keyval.get("buildingfile")?),
                        name: keyval.get("propertyname")?.to_string(),
                        shadow_flag: keyval.get("shadowflag")?.to_string(),
                    })
                ))
            },
            _ => None
        }
    }
}
