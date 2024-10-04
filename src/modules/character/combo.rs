use crate::modules::conversion::common::parse_group_line_as_f32;

pub struct Combos {
    pub combos: Vec<Combo>,
    cur_combo_index: u32,
    is_pre_input: bool,
    is_next_pre_input: bool,
}

impl Combos {

    // pub fn combo_process(&mut self) {
    //     if self.cur_combo_index != 0 {
    //         if let Some(motion_data) = &self.cur_race_motion_data {
    //             let elapsed_time = self.get_attacking_elapsed_time();

    //             if self.is_pre_input {
    //                 if elapsed_time > motion_data.get_next_combo_time() {
    //                     self.run_next_combo();
    //                     self.is_pre_input = false;
    //                     return;
    //                 }
    //             }
    //         } else {
    //             println!("Attacking motion data is NULL!");
    //             self.clear_combo();
    //             return;
    //         }
    //     } else {
    //         self.is_pre_input = false;

    //         if !self.is_using_skill() && self.is_next_pre_input {
    //             self.run_next_combo();
    //             self.is_next_pre_input = false;
    //         }
    //     }
    // }

    fn run_next_combo(&mut self) {
        self.cur_combo_index += 1;
    }

    fn clear_combo(&mut self) {
        self.cur_combo_index = 0;
        self.is_pre_input = false;
        // self.cur_race_motion_data = None;
    }
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