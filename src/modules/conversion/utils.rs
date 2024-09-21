use image::{ImageBuffer, Luma};

pub fn generate_tiles_atlas(tiles: Vec<ImageBuffer<Luma<u8>, Vec<u8>>>) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    const TILE_SIZE: u32 = 256;
    const ATLAS_SIZE: u32 = TILE_SIZE * 3;
    let mut atlas = ImageBuffer::new(ATLAS_SIZE, ATLAS_SIZE);
    let tile_size = 256;
    let tiles_per_row = ATLAS_SIZE / tile_size;
    for (index, tile) in tiles.iter().enumerate() {
        let x = (index as u32 % tiles_per_row) * tile_size;
        let y = (index as u32 / tiles_per_row) * tile_size;
        for (tile_x, tile_y, pixel) in tile.enumerate_pixels() {
            if x + tile_x < ATLAS_SIZE && y + tile_y < ATLAS_SIZE {
                atlas.put_pixel(x + tile_x, y + tile_y, *pixel);
            }
        }
    }
    atlas
}