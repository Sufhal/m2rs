use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text}, BrushBuilder, TextBrush};

pub struct UserInterface<'a> {
    pub brush: TextBrush<FontRef<'a>>,
    pub section: TextSection<'a>,
}

impl<'a> UserInterface<'a> {
    pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
        let font = include_bytes!("../../fonts/JetBrainsMono-Regular.ttf");

        let brush = BrushBuilder::using_font_bytes(font).unwrap()
         /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(device, surface_configuration.width, surface_configuration.height, wgpu::TextureFormat::Bgra8UnormSrgb);
        
        // Directly implemented from glyph_brush.
        let section = TextSection::default().add_text(Text::new("Hello World"));

        Self {
            brush,
            section
        }
    }
}