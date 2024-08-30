// const std::string & c_rstrFileName	= rVector[0].c_str();
// const std::string & c_rstrUScale	= rVector[1].c_str();
// const std::string & c_rstrVScale	= rVector[2].c_str();
// const std::string & c_rstrUOffset	= rVector[3].c_str();
// const std::string & c_rstrVOffset	= rVector[4].c_str();
// const std::string & c_rstrbSplat	= rVector[5].c_str();
// const std::string & c_rstrBegin		= rVector[6].c_str();
// const std::string & c_rstrEnd		= rVector[7].c_str();

use crate::modules::{assets::assets::{load_png_bytes, load_string, load_texture}, core::texture::Texture};


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChunkTextureSet {
    pub textures: Vec<u8>
}

impl ChunkTextureSet {
    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let filename = format!("{path}/textureset.json");
        let file = load_string(&filename).await?;
        let setting = serde_json::from_str::<Self>(&file)?;
        Ok(setting)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextureSet {
    pub definitions: Vec<TextureDefinition>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TextureDefinition {
    pub file: String,
    pub u_scale: f32,
    pub v_scale: f32
}

impl TextureSet {

    pub async fn read(path: &str) -> anyhow::Result<Self> {
        let filename = format!("{path}/textureset.json");
        let file = load_string(&filename).await?;
        let setting = serde_json::from_str::<Self>(&file)?;
        Ok(setting)
    }

    pub async fn load_textures(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Vec<Texture>> {
        let mut textures = Vec::new();
        for definition in &self.definitions {
            let texture = definition.load(device, queue).await?;
            textures.push(texture);
        }
        Ok(textures)
    }

    pub async fn load_bytes(&self) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut textures = Vec::new();
        for definition in &self.definitions {
            let texture = definition.load_bytes().await?;
            textures.push(texture);
        }
        Ok(textures)
    }
    
    pub fn from_txt(data: &str) -> Self {
        let mut lines = data.lines().map(str::trim);
        let mut definitions = Vec::new();
        lines.next(); // TextureSet
        lines.next(); // TextureCount
        while let Some(line) = lines.next() {
            if line.starts_with("Start Texture") {
                let file = lines.next().unwrap()
                    .trim_matches('"')
                    .replace("\\", "/")
                    .replace("d:/ymir work", "pack")
                    .replace(".dds", ".png")
                    .to_string();
                let u_scale = lines.next().unwrap()
                    .parse::<f32>().unwrap();
                let v_scale = lines.next().unwrap()
                    .parse::<f32>().unwrap();
                for _ in 0..5 {
                    lines.next();
                }
                definitions.push(TextureDefinition {
                    file,
                    u_scale,
                    v_scale,
                });
            }
        }
        TextureSet { definitions }
    }
}

impl TextureDefinition {
    pub async fn load(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> anyhow::Result<Texture> {
        load_texture(&self.file, device, queue).await
    }
    pub async fn load_bytes(&self) -> anyhow::Result<Vec<u8>> {
        load_png_bytes(&self.file).await
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// ````
/// 
/// [7, 0] < index of texture passed in the shader
///  ^ index of texture defined in the tile (the index of the textureset.json)
/// ``````
pub struct TextureSetUniform {
    texture_set: [u32; 32], 
}

impl TextureSetUniform {
    pub fn from(indices: &Vec<u8>) -> Self {
        let mut texture_set = [0; 32];
        for (i, index) in indices.iter().enumerate().take(16) {
            texture_set[i * 2]      = *index as u32;
            texture_set[i * 2 + 1]  = i as u32;
        }
        Self { texture_set }
    } 
}
