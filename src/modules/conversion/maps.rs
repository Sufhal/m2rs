use std::fs;
use std::path::Path;
use image::GrayImage;
use crate::modules::conversion::common::write;
use crate::modules::conversion::utils::generate_tiles_atlas;
use crate::modules::terrain::areadata::AreaData;
use crate::modules::terrain::texture_set::{ChunkTextureSet, TextureSet};

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

                                // areadata
                                if let Ok(areadata) = fs::read_to_string(Path::new(&format!("{}/areadata.txt", element.path().to_str().unwrap()))) {
                                    let parsed = AreaData::from_txt(&areadata);
                                    write(
                                        &format!("{}/areadata.json", element.path().to_str().unwrap()), 
                                        serde_json::to_string(&parsed).unwrap()
                                    );
                                }

                                let tile_indices = fs::read(&format!("{}/tile.raw", element.path().to_str().unwrap())).unwrap();

                                // Colors are counted because only 8 textures are allowed in a chunk
                                // The under-used colors will be replaced with nearest color
                                let mut colors: Vec<(u8, u64)> = Vec::new();

                                // Delete first and last row and column of tile map
                                let mut tile_indices = tile_indices
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
                                        if let Some(tuple) = colors.iter_mut().find(|(v, _count)| *v == real_index) {
                                            tuple.1 += 1;
                                        } else {
                                            colors.push((real_index, 1));
                                        }
                                        real_index
                                        // colors.iter().position(|(v, _)| *v == real_index).unwrap() as u8
                                    })
                                    .collect::<Vec<_>>();

                                while colors.len() > 8 {
                                    // Maximum colors allowed in a chunk is 8. We needs to replace the under-used colors with its nearest color
                                    let under_used_color_index = colors
                                        .iter()
                                        .enumerate()
                                        .fold(0, |mut acc, (idx, v)| {
                                            if v.1 < colors[acc].1 {
                                                acc = idx;
                                            }
                                            acc
                                        });
                                    let color_to_merge = colors[under_used_color_index].0;
                                    
                                    // Find the nearest colors
                                    let mut near_colors: Vec<(u8, u64)> = Vec::new();
                                    let add_to_near_colors = |near_colors: &mut Vec<(u8, u64)>, near_color: u8| {
                                        if let Some(tuple) = near_colors.iter_mut().find(|(v, _count)| *v == near_color) {
                                            tuple.1 += 1;
                                        } else {
                                            near_colors.push((near_color, 1));
                                        }
                                    };
                                    let size = 256.0;
                                    for i in 0..tile_indices.len() {
                                        let color = tile_indices[i];
                                        let y = f64::floor(i as f64 / size) as u64;
                                        let x = (i as f64 % size) as u64;
                                        if color != color_to_merge {
                                            continue;
                                        }
                                        if x > 0 {
                                            // [ ][ ][ ]
                                            // [x][o][ ]
                                            // [ ][ ][ ]
                                            let pos = i - 1;
                                            if tile_indices[i - 1] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if x < size as u64 {
                                            // [ ][ ][ ]
                                            // [ ][o][x]
                                            // [ ][ ][ ]
                                            let pos = i + 1;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if y > 0 {
                                            // [ ][x][ ]
                                            // [ ][o][ ]
                                            // [ ][ ][ ]
                                            let pos = i - size as usize;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if x < size as u64 {
                                            // [ ][ ][ ]
                                            // [ ][o][ ]
                                            // [ ][x][ ]
                                            let pos = i + size as usize;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if y > 0 && x > 0 {
                                            // [x][ ][ ]
                                            // [ ][o][ ]
                                            // [ ][ ][ ]
                                            let pos = i - size as usize - 1;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if y < size as u64 && x > 0 {
                                            // [ ][ ][ ]
                                            // [ ][o][ ]
                                            // [x][ ][ ]
                                            let pos = i + size as usize - 1;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if y < size as u64 && x < size as u64 {
                                            // [ ][ ][ ]
                                            // [ ][o][ ]
                                            // [ ][ ][x]
                                            let pos = i + size as usize + 1;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                        if y < size as u64 && x < size as u64 {
                                            // [ ][ ][x]
                                            // [ ][o][ ]
                                            // [ ][ ][ ]
                                            let pos = i - size as usize + 1;
                                            if tile_indices[pos] != color_to_merge {
                                                add_to_near_colors(&mut near_colors, tile_indices[pos]);
                                            }
                                        }
                                    }

                                    let most_present_near_color_index = near_colors
                                        .iter()
                                        .enumerate()
                                        .fold(0, |mut acc, (idx, v)| {
                                            if v.1 > colors[acc].1 {
                                                acc = idx;
                                            }
                                            acc
                                        });

                                    // Rewrite the tile 
                                    for i in 0..tile_indices.len() {
                                        if color_to_merge == tile_indices[i] {
                                            tile_indices[i] = near_colors[most_present_near_color_index].0;
                                        }
                                    }

                                    // Update actual colors
                                    colors.remove(colors.iter().position(|(v, _)| *v == color_to_merge).unwrap());
                                }

                                let mut tiles = Vec::new();
                                for i in 0..colors.len() {
                                    let (index, _) = colors.get(i).unwrap();
                                    let alpha_map = tile_indices
                                        .iter()
                                        .fold(Vec::new(), |mut acc, v| {
                                            if *v == *index {
                                                acc.push(u8::MAX);
                                            } else {
                                                acc.push(u8::MIN);
                                            }
                                            acc
                                        });
                                    let blurred_alpha_map: GrayImage = image::ImageBuffer::from_raw(256, 256, alpha_map).unwrap();
                                    use image::imageops::blur;
                                    let blurred_alpha_map = blur(&blurred_alpha_map, 1.0);
                                    write(
                                        &format!("{}/tile_{i}.raw", element.path().to_str().unwrap()), 
                                        blurred_alpha_map.to_vec()
                                    );
                                    tiles.push(blurred_alpha_map); 
                                }
                                let tiles_atlas = generate_tiles_atlas(tiles);
                                write(
                                    &format!("{}/tiles_atlas.raw", element.path().to_str().unwrap()), 
                                    tiles_atlas.to_vec()
                                );

                                write(
                                    &format!("{}/textureset.json", element.path().to_str().unwrap()), 
                                    serde_json::to_string(&ChunkTextureSet { textures: colors.iter().map(|(v, _)| *v).collect() }).unwrap()
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