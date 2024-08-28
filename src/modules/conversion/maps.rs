use std::fs;
use std::path::Path;

use crate::modules::terrain::texture_set::TextureSet;

pub fn convert_maps() {
    let maps_directory = Path::new("assets/pack/map");
    let maps = fs::read_dir(maps_directory).unwrap();
    for map in maps {
        if let Ok(map) = map {
            if let Ok(texture_set) = fs::read_to_string(Path::new(&format!("{}/textureset.txt", map.path().to_str().unwrap()))) {
                let parsed = TextureSet::from_txt(&texture_set);
                fs::write(&format!("{}/textureset.json", map.path().to_str().unwrap()), serde_json::to_string(&parsed).unwrap()).unwrap();
            }
        }
    }
}