use std::fs;
use std::path::Path;

use image::GrayImage;

use crate::modules::conversion::common::write;
use crate::modules::terrain::texture_set::{ChunkTextureSet, TextureSet};
use crate::modules::utils::structs::Set;

pub fn convert_maps() {
    let maps_directory = Path::new("assets/pack/map");
    let maps = fs::read_dir(maps_directory).unwrap();
    for map in maps {
        if let Ok(map) = map {
            if map.file_type().unwrap().is_dir() {
                for element in fs::read_dir(map.path()).unwrap() {
                    if let Ok(element) = element {
                        println!("Reading \"{}\"", element.path().to_str().unwrap());
                        if element.file_name() == "textureset.txt" {
                            // textureset.txt
                            if let Ok(texture_set) = fs::read_to_string(Path::new(&format!("{}/textureset.txt", map.path().to_str().unwrap()))) {
                                let parsed = TextureSet::from_txt(&texture_set);
                                write(
                                    &format!("{}/textureset.json", map.path().to_str().unwrap()), 
                                    serde_json::to_string(&parsed).unwrap()
                                );
                            }
                        }
                        else {
                            if element.file_type().unwrap().is_dir() {
                                // this path is a chunk
                                let tile_indices = fs::read(&format!("{}/tile.raw", element.path().to_str().unwrap())).unwrap();

                                // Delete first and last row and column of tile map
                                // Replace its values with actual textures indices passed in the shader
                                let mut textures_set = Set::new();
                                let tile_indices = tile_indices
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| {
                                        const ORIGINAL_SIZE: f64 = 258.0;
                                        let line_index = f64::floor(*i as f64 / ORIGINAL_SIZE) as u64;
                                        let colmun_index = (*i as f64 % ORIGINAL_SIZE) as u64;
                                        let ignore = line_index == 0 || line_index == 257 || colmun_index == 0 || colmun_index == 257;
                                        !ignore
                                    })
                                    .map(|(_, v)| {
                                        let real_index = (*v) - 1;
                                        textures_set.insert(real_index);
                                        textures_set.position(&real_index).unwrap() as u8
                                    })
                                    .collect::<Vec<_>>();

                                for i in 0..textures_set.len() {
                                    let index = textures_set.get(i).unwrap();
                                    let alpha_map = tile_indices
                                        .iter()
                                        .fold(Vec::<u8>::new(), |mut acc, v| {
                                            if *v == *index {
                                                acc.push(u8::MAX);
                                            } else {
                                                acc.push(u8::MIN);
                                            }
                                            acc
                                        });
                                    let blurred_alpha_map: GrayImage = image::ImageBuffer::from_raw(256, 256, alpha_map).unwrap();
                                    use image::imageops::blur;
                                    let blurred_alpha_map = blur(&blurred_alpha_map, 2.0);
                                    write(
                                        &format!("{}/tile_{i}.raw", element.path().to_str().unwrap()), 
                                        blurred_alpha_map.to_vec()
                                    );
                                    
                                }

                                write(
                                    &format!("{}/textureset.json", element.path().to_str().unwrap()), 
                                    serde_json::to_string(&ChunkTextureSet { textures: textures_set.to_vec() }).unwrap()
                                );
                            }
                        }
                    }
                } 
            }
        }
    }
    println!("Done ✨");
}