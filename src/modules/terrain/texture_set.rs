// const std::string & c_rstrFileName	= rVector[0].c_str();
// const std::string & c_rstrUScale	= rVector[1].c_str();
// const std::string & c_rstrVScale	= rVector[2].c_str();
// const std::string & c_rstrUOffset	= rVector[3].c_str();
// const std::string & c_rstrVOffset	= rVector[4].c_str();
// const std::string & c_rstrbSplat	= rVector[5].c_str();
// const std::string & c_rstrBegin		= rVector[6].c_str();
// const std::string & c_rstrEnd		= rVector[7].c_str();


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
    fn from_str(data: &str) -> Self {
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


