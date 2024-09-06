use crate::modules::environment::environment::MsEnv;
use super::common::write;

pub fn convert_environments() {
    let dir = std::path::Path::new("assets/pack/environment");
    let entries = std::fs::read_dir(dir).unwrap();
    for entry in entries {
        if let Ok(entry) = entry {
            if entry.file_type().unwrap().is_dir() { continue }
            let filename = entry.file_name();
            let filename = filename.to_str().unwrap();
            if !filename.ends_with(".msenv") { continue }
            let name = filename.split('.').collect::<Vec<_>>()[0];
            if entry.file_name().to_str().unwrap().ends_with(".msenv") && entry.file_type().unwrap().is_file() {
                if let Ok(string) = std::fs::read_to_string(entry.path()) {
                    let msenv = MsEnv::from_txt(&string);
                    write(
                        &format!("{}/{name}.json", dir.to_str().unwrap()), 
                        serde_json::to_string(&msenv).unwrap()
                    );
                }
            }
        }
    }
}