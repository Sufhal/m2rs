use crate::modules::{assets::assets::load_string, conversion::common::bye_ymir, state::State, utils::functions::{correct_color, srgb_to_linear}};

use super::{clouds::Clouds, cycle::Cycle, fog::Fog, sky::Sky, sun::Sun};

pub struct Environment {
    pub cycle: Cycle,
    pub fog: Fog,
    pub sun: Sun,
    pub sky: Sky,
    pub clouds: Clouds,
}

impl Environment {

    pub async fn load(name: &str, state: &State<'_>) -> anyhow::Result<Self> {
        let cycle = Cycle::new();
        let day_msenv = MsEnv::read(name).await?;
        let night_msenv = MsEnv::read("moonlight04").await?;
        Ok(Self {
            cycle,
            fog: Fog::new(&day_msenv, &night_msenv),
            sun: Sun::new(&day_msenv, &night_msenv, state),
            sky: Sky::new(&day_msenv, &night_msenv, state),
            clouds: Clouds::new(&day_msenv, state).await?
        })
    }

    pub fn update(&mut self, delta: f32, queue: &wgpu::Queue) {
        self.cycle.update(delta);
        self.sun.update(&self.cycle, queue);
        self.clouds.update(&self.cycle, queue);
    }

}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MsEnv {
    pub directional_light: DirectionalLight,
    pub material: Material,
    pub fog: FogData,
    pub sky_box: SkyBox
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DirectionalLight {
    pub direction: [f32; 3],
    pub background: Light,
    pub character: Light,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Light {
    pub enable: bool,
    pub diffuse: [f32; 4],
    pub ambient: [f32; 4]
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Material {
    pub diffuse: [f32; 4],
    pub ambient: [f32; 4],
    pub emissive: [f32; 4],
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FogData {
    pub enable: bool,
    pub near: f32,
    pub far: f32,
    pub color: [f32; 4]
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SkyBox {
    pub scale: [f32; 3],
    pub gradient_level_upper: u8,
    pub gradient_level_lower: u8,
    pub gradient: [[f32; 4]; 6],
    pub cloud_scale: [f32; 2],
    pub cloud_height: f32,
    pub cloud_texture_scale: [f32; 2],
    pub cloud_speed: [f32; 2],
    pub cloud_texture_file: String,
    pub cloud_colors: [[f32; 4]; 2],
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
            gradient: [[0.0; 4]; 6],
            cloud_scale: [0.0; 2],
            cloud_height: 0.0,
            cloud_texture_scale: [0.0; 2],
            cloud_speed: [0.0; 2],
            cloud_texture_file: String::new(),
            cloud_colors: [[0.0; 4]; 2],
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
                    "Background" => directional_light.background.diffuse = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    "Character" => directional_light.character.diffuse = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    "Material" => material.diffuse = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    _ => {}
                },
                "Ambient" => match current_group.as_str() {
                    "Background" => directional_light.background.ambient = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    "Character" => directional_light.character.ambient = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    "Material" => material.ambient = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    _ => {}
                },
                "Emissive" => {
                    material.emissive = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]);
                }
                "NearDistance" => {
                    fog.near = parts[1].parse::<f32>().unwrap() / 100.0;
                }
                "FarDistance" => {
                    fog.far = parts[1].parse::<f32>().unwrap() / 100.0;
                }
                "Color" => match current_group.as_str() {
                    "Fog" => fog.color = correct_color([
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ]),
                    _ => {}
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
                "CloudScale" => {
                    sky_box.cloud_scale = [
                        parts[1].parse::<f32>().unwrap() / 100.0,
                        parts[2].parse::<f32>().unwrap() / 100.0,
                    ];
                }
                "CloudHeight" => {
                    sky_box.cloud_height = parts[1].parse::<f32>().unwrap() / 100.0;
                }
                "CloudSpeed" => {
                    sky_box.cloud_speed = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                    ];
                }
                "CloudTextureScale" => {
                    sky_box.cloud_texture_scale = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                    ];
                }
                "CloudTextureFileName" => {
                    sky_box.cloud_texture_file = {
                        let mut parts = parts.clone();
                        parts.remove(0);
                        bye_ymir(&parts.join(""))
                    };
                }
                "List" => match parts[1] {
                    "CloudColor" => (),
                    "Gradient" => {
                        let to_colors = |line: &str| {
                            let mut colors = line.split_whitespace().map(|v| v.parse().unwrap_or(0.0)).collect::<Vec<f32>>();
                            while colors.len() < 4 {
                                colors.push(0.0);
                            }
                            colors
                        };
                        lines.next();
                        let c0 = to_colors(lines.next().unwrap());
                        lines.next();
                        lines.next();
                        let c1 = to_colors(lines.next().unwrap());
                        lines.next();
                        lines.next();
                        let c2 = to_colors(lines.next().unwrap());
                        lines.next();
                        lines.next();
                        let c3 = to_colors(lines.next().unwrap());
                        lines.next();
                        lines.next();
                        let c4 = to_colors(lines.next().unwrap());
                        let c5 = to_colors(lines.next().unwrap());

                        let to_adapted = |v: Vec<f32>| {
                            let c = srgb_to_linear([v[0], v[1], v[2]]);
                            [c[0], c[1], c[2], 1.0]
                        };
                        sky_box.gradient = [
                            to_adapted(c0),
                            to_adapted(c1),
                            to_adapted(c2),
                            to_adapted(c3),
                            to_adapted(c4),
                            to_adapted(c5),
                        ];
                    }
                    _ => {}
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