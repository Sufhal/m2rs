use crate::modules::character::combo::{Combo, ComboInput, Combos};

use super::common::write;

pub fn convert_msa() {
    // pc
    let dir = std::path::Path::new("assets/pack/pc");
    let entries = std::fs::read_dir(dir).unwrap();
    for entry in entries.map(|v| v.ok()).flatten() {
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }
        let race = entry.file_name();
        let race = race.to_str().unwrap_or("");
        let dir = format!("assets/pack/pc/{race}");
        let path = std::path::Path::new(&dir);
        let entries = std::fs::read_dir(path).unwrap();
        for entry in entries.map(|v| v.ok()).flatten() {
            if !entry.file_type().unwrap().is_dir() {
                continue;
            }
            let race_subpath = entry.file_name();
            let race_subpath = race_subpath.to_str().unwrap_or("");
            let dir = format!("assets/pack/pc/{race}/{race_subpath}");
            let path = std::path::Path::new(&dir);
            let entries = std::fs::read_dir(path).unwrap();
            let mut combo_inputs = Vec::new();
            for _ in 0..10 {
                combo_inputs.push(None);
            }
            for entry in entries.map(|v| v.ok()).flatten() {
                let filename = entry.file_name();
                let filename = filename.to_str().unwrap_or("");
                if !filename.ends_with(".msa") {
                    continue;
                }
                if filename.starts_with("combo_") {
                    let index = filename
                        .split("_")
                        .collect::<Vec<_>>()[1]
                        .split(".")
                        .collect::<Vec<_>>()[0]
                        .parse::<usize>()
                        .unwrap();
                    combo_inputs[index - 1] = Some(ComboInput::from_txt(&std::fs::read_to_string(entry.path()).unwrap()));
                }
            }
            let combo_inputs = combo_inputs
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            if combo_inputs.len() > 0 {
                let combos = combo_inputs
                    .into_iter()
                    .map(|input| Combo { input })
                    .collect::<Vec<Combo>>();
                write(&format!("{dir}/combos.json"), serde_json::to_string(&combos).unwrap());
            }
        }
    }
}