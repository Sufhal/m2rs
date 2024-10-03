use std::fs::FileType;

use crate::modules::character::combo::ComboInput;

pub fn convert_msa() {
    // pc
    let dir = std::path::Path::new("assets/pack/pc");
    let entries = std::fs::read_dir(dir).unwrap();
    for entry in entries.map(|v| v.ok()).flatten() {
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let race = entry.file_name().to_str().unwrap_or("");
        let dir = std::path::Path::new(&format!("assets/pack/pc/{race}"));
        let entries = std::fs::read_dir(dir).unwrap();
        for entry in entries.map(|v| v.ok()).flatten() {
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let race_subpath = entry.file_name().to_str().unwrap_or("");
            let dir = std::path::Path::new(&format!("assets/pack/pc/{race}/{race_subpath}"));
            let entries = std::fs::read_dir(dir).unwrap();

            let mut combo_inputs = Vec::new();
            for entry in entries.map(|v| v.ok()).flatten() {
                let filename = entry.file_name().to_str().unwrap_or("");
                if !filename.ends_with(".msa") {
                    continue;
                }
                if filename.starts_with("combo_") {
                    combo_inputs.push(ComboInput::from_txt(&std::fs::read_to_string(entry.path()).unwrap()));
                }
            }
        }
    }
}