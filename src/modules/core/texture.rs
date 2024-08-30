use std::path::Path;
use anyhow::*;
use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::modules::{state::State, utils::functions::is_power_of_two};

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {

    pub fn from_raw_bytes(
        bytes: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        bytes_per_row: u32,
        state: &State<'_>
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = state.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &bytes,
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
        state.queue.submit(Some(encoder.finish()));
        Self {
            texture,
            view,
            sampler
        }
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let debug = false;
        if debug {
            let filename = if label.ends_with(".png") || label.ends_with(".jpg") {
                format!("trash/{label}")
            } else {
                format!("trash/{label}.png")
            };
            let _ = std::fs::write(Path::new(&filename), bytes);
        }
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.
    
    pub fn create_depth_texture(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, label: &str) -> Self {
        let size = wgpu::Extent3d { // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor { // 4.
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                compare: Some(wgpu::CompareFunction::LessEqual), // 5.
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.0,
                ..Default::default()
            }
        );

        Self { texture, view, sampler }
    }
}

#[allow(dead_code)]
pub struct TextureAtlas {
    pub texture: Texture,
}

impl TextureAtlas {
    /// # Unfinished
    /// I'm sending directly the textures in the shader (terrain) because each textures have a different size.
    /// Already borred of implementing a solution to resolve this problem.
    /// Create a new TextureAtlas using raw bytes. 
    /// Each raw_bytes slices should represent an **4 channels** (rgba) texture with a size that is **power of two**.
    pub fn new(raw_bytes: Vec<Vec<u8>>, state: &State<'_>) -> Self {
        if raw_bytes.len() == 0 {
            panic!("raw_bytes.len() must be > 0 to create a TextureAtlas");
        }
        if !is_power_of_two(raw_bytes[0].len() as u64) {
            panic!("raw_bytes[0] must be square power of two");
        }
        let textures_count = raw_bytes.len() as u64;
        let tile_size = f64::sqrt(raw_bytes[0].len() as f64) as u64;
        let pixels = tile_size * tile_size * textures_count;
        let atlas_size = (f64::sqrt(pixels as f64) as u64).next_power_of_two();
        let tiles_per_row = atlas_size / tile_size;

        let mut atlas_raw_bytes = Vec::new();

        for x in 0..atlas_size {
            for y in 0..atlas_size {
                let tile_x = (x / tile_size) as i64;
                let tile_y = (y / tile_size) as i64;
                let index = (tile_x + tile_y * tiles_per_row as i64) as usize;

                if index < textures_count as usize {
                    let local_x = x - (tile_x as u64 * tile_size);
                    let local_y = y - (tile_y as u64 * tile_size);
                    let local_index = (local_x + local_y * tile_size) as usize;

                    // println!("{textures_count} {tile_size} {atlas_size} {tiles_per_row} : {x} - {y} : {local_x} {local_x} {local_index}");
                    atlas_raw_bytes.extend([
                        raw_bytes[index][local_index + 0],
                        raw_bytes[index][local_index + 1],
                        raw_bytes[index][local_index + 2],
                        raw_bytes[index][local_index + 3],
                    ]);
                }
                else {
                    atlas_raw_bytes.extend([0, 0, 0, 0]);
                }
            }
        }

        let size_u32 = atlas_size as u32;

        TextureAtlas {
            texture: Texture::from_raw_bytes(
                &atlas_raw_bytes, 
                size_u32, 
                size_u32, 
                wgpu::TextureFormat::Rgba8UnormSrgb, 
                4 * size_u32, 
                state
            )
        }
    }
}  