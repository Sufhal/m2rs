use crate::modules::conversion::common::parse_group_line_as_f32;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Combos {
    pub combos: Vec<Combo>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Combo {
    pub input: ComboInput,
}


#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ComboInput {
    pub pre_input_time: f32,
    pub direct_input_time: f32,
    pub input_limit_time: f32,
}

impl Default for ComboInput {
    fn default() -> Self {
        Self {
            pre_input_time: 0.0,
            direct_input_time: 0.0,
            input_limit_time: 0.0,
        }
    }
}

impl ComboInput {
    pub fn from_txt(data: &str) -> Self {
        let mut input = Self::default();
        let mut lines = data.lines().map(str::trim);
        while let Some(line) = lines.next() {
            if line.starts_with("Group ComboInputData") {
                lines.next();
                input.pre_input_time = parse_group_line_as_f32(lines.next().unwrap());
                input.direct_input_time = parse_group_line_as_f32(lines.next().unwrap());
                input.input_limit_time = parse_group_line_as_f32(lines.next().unwrap());
            }
        }
        input
    }
}