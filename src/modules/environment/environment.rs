use crate::modules::{assets::assets::load_string, state::State};

use super::{cycle::Cycle, fog::Fog, sky::Sky, sun::Sun};

pub struct Environment {
    pub cycle: Cycle,
    pub fog: Fog,
    pub sun: Sun,
    // pub sky: Sky,
}

impl Environment {

    pub async fn load(name: &str, state: &State<'_>) -> anyhow::Result<Self> {
        let cycle = Cycle::new();
        let msenv = MsEnv::read(name).await?;
        let vec3 = |v: [f32; 4]| [v[0], v[1], v[2]];
        Ok(Self {
            cycle,
            fog: Fog::new(
                msenv.fog.near, 
                msenv.fog.far, 
                vec3(msenv.fog.color)
            ),
            sun: Sun::new(
                &msenv,
                state
            ),
            // sky: Sky::new(
            //     msenv.sky_box.cloud_gradient
            // )
        })
    }

    pub fn update(&mut self, delta: f32, queue: &wgpu::Queue) {
        self.cycle.update(delta);
        self.sun.update(&self.cycle, queue);
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
    pub gradient: [[f32; 4]; 5],
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
            gradient: [[0.0; 4]; 5],
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
                "Color" => match current_group.as_str() {
                    "Fog" => fog.color = [
                        parts[1].parse().unwrap(),
                        parts[2].parse().unwrap(),
                        parts[3].parse().unwrap(),
                        parts[4].parse().unwrap(),
                    ],
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
                "CloudTextureFileName" => {
                    sky_box.cloud_texture_file = parts[1].to_string();
                }
                "List" => match parts[1] {
                    "CloudColor" => (),
                    "Gradient" => {
                        let to_colors = |line: &str| line.split_whitespace().map(|v| v.parse().unwrap()).collect::<Vec<f32>>();
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
                        sky_box.gradient = [
                            [c0[0], c0[1], c0[2], c0[3]],
                            [c1[0], c1[1], c1[2], c1[3]],
                            [c2[0], c2[1], c2[2], c2[3]],
                            [c3[0], c3[1], c3[2], c3[3]],
                            [c4[0], c4[1], c4[2], c4[3]],
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