use crate::modules::core::{light_source::LightSource};

// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/shadow/shader.wgsl
// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/shadow/main.rs#L750

pub struct ShadowTest {
    texture: wgpu::Texture,
    pub light: LightSource,
}

impl ShadowTest {
    pub fn new(device: &wgpu::Device) -> Self {

        let extent = wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });
        
        // Créer une vue sur la shadow map pour l'utiliser dans les shaders
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            light: LightSource::new([0.0, 0.0, 0.0], view)
        }
    }
}