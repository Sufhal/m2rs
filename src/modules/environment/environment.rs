use crate::modules::assets::assets::load_string;

use super::{fog::Fog, sky::Sky, sun::Sun};

pub struct Environment {
    pub fog: Fog,
    pub sky: Sky,
    pub sun: Sun,
}

impl Environment {

    pub async fn load(name: &str) -> anyhow::Result<Self> {
        let msenv = MsEnv::read(name).await?;
        todo!()
    }

}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MsEnv {
    directional_light: DirectionalLight,
    material: Material,
    fog: FogData,
    sky_box: SkyBox
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DirectionalLight {
    direction: [f32; 3],
    background: Light,
    character: Light,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Light {
    enable: bool,
    diffuse: [f32; 4],
    ambient: [f32; 4]
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Material {
    diffuse: [f32; 4],
    ambient: [f32; 4],
    emissive: [f32; 4],
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct FogData {
    enable: bool,
    near: f32,
    far: f32,
    color: [f32; 4]
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SkyBox {
    scale: [f32; 3],
    gradient_level_upper: u8,
    gradient_level_lower: u8,
    cloud_scale: [f32; 2],
    cloud_height: f32,
    cloud_texture_scale: [f32; 2],
    cloud_speed: [f32; 2],
    cloud_texture_file: String,
    cloud_colors: [[f32; 4]; 2],
    cloud_gradient: [[f32; 4]; 10]
}

impl MsEnv {

    pub async fn read(name: &str) -> anyhow::Result<Self> {
        let file = load_string(&format!("pack/environment/{name}.json")).await?;
        Ok(serde_json::from_str::<Self>(&file)?)
    }

    pub fn from_txt(data: &str) -> Self {

        let mut directional_light = DirectionalLight {
            direction: [0.0; 3],
            background: Light {
                enable: false,
                diffuse: [0.0; 4],
                ambient: [0.0; 4],
            },
            character: Light {
                enable: false,
                diffuse: [0.0; 4],
                ambient: [0.0; 4],
            },
        };
    
        let mut material = Material {
            diffuse: [0.0; 4],
            ambient: [0.0; 4],
            emissive: [0.0; 4],
        };
    
        let mut fog = FogData {
            enable: false,
            near: 0.0,
            far: 0.0,
            color: [0.0; 4],
        };
    
        let mut sky_box = SkyBox {
            scale: [0.0; 3],
            gradient_level_upper: 0,
            gradient_level_lower: 0,
            cloud_scale: [0.0; 2],
            cloud_height: 0.0,
            cloud_texture_scale: [0.0; 2],
            cloud_speed: [0.0; 2],
            cloud_texture_file: String::new(),
            cloud_colors: [[0.0; 4]; 2],
            cloud_gradient: [[0.0; 4]; 10],
        };
    
        let mut current_group = String::new();
        let mut lines = data.lines().map(str::trim);

        while let Some(line) = lines.next() {

            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.is_empty() {
                continue;
            }
    
            match parts[0] {
                "Group" => {
                    current_group = parts[1].to_string();
                }
                "Direction" => {
                    directional_light.direction = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                    ];
                }
                "Enable" => match current_group.as_str() {
                    "Background" => directional_light.background.enable = parts[1] == "1",
                    "Character" => directional_light.character.enable = parts[1] == "1",
                    "Fog" => fog.enable = parts[1] == "1",
                    "Filter" => {} // Ignorer pour l'instant
                    "LensFlare" => {} // Ignorer pour l'instant
                    _ => {}
                },
                "Diffuse" => match current_group.as_str() {
                    "Background" => directional_light.background.diffuse = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    "Character" => directional_light.character.diffuse = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    "Material" => material.diffuse = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    _ => {}
                },
                "Ambient" => match current_group.as_str() {
                    "Background" => directional_light.background.ambient = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    "Character" => directional_light.character.ambient = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    "Material" => material.ambient = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
                    _ => {}
                },
                "Emissive" => {
                    material.emissive = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ];
                }
                "NearDistance" => {
                    fog.near = parts[1].parse().unwrap();
                }
                "FarDistance" => {
                    fog.far = parts[1].parse().unwrap();
                }
                "Color" => {
                    fog.color = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ];
                }
                "Scale" => match current_group.as_str() {
                    "SkyBox" => {
                        sky_box.scale = [
                            parts[1].parse().unwrap(),
                            parts[2].parse().unwrap(),
                            parts[3].parse().unwrap(),
                        ];
                    }
                    _ => {}
                }
                "GradientLevelUpper" => {
                    sky_box.gradient_level_upper = parts[1].parse().unwrap();
                }
                "GradientLevelLower" => {
                    sky_box.gradient_level_lower = parts[1].parse().unwrap();
                }
                "CloudTextureFileName" => {
                    sky_box.cloud_texture_file = parts[1].to_string();
                }
                _ => {}
            }
        }
        Self {
            directional_light,
            material,
            fog,
            sky_box,
        }
    }
}