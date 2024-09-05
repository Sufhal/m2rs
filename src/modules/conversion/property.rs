use std::path::Path;
use std::fs;
use crate::modules::terrain::property::{Properties, Property};

use super::common::{search_files_in_directory, write};

pub fn convert_property() {
    let directory = Path::new("assets/pack/property");
    let mut files = Vec::new();
    search_files_in_directory(directory, &mut files).unwrap();

    let mut properties = Vec::new();
    for file in files {
        if let Ok(string) = fs::read_to_string(file) {
            if let Some(property) = Property::from_txt(&string) {
               properties.push(property);
            }
        }
    }

    write(
        &format!("{}/properties.json", directory.to_str().unwrap()), 
        serde_json::to_string(&Properties { properties }).unwrap()
    );
}