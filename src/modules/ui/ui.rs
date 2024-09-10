use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text}, BrushBuilder, TextBrush};
use crate::modules::utils::{functions::{calculate_fps, to_fixed_2, u8_to_string_with_len}, structs::LimitedVec};

pub struct UserInterface {
    pub brush: TextBrush<FontRef<'static>>,
    pub informations: Informations,
    pub metrics: Metrics,
    pub std_out: LimitedVec<String>,
    scale_factor: f32,
    elapsed_time: f64,
}

impl UserInterface {

    pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration, multisampled_texture: &wgpu::Texture, scale_factor: f32) -> Self {
        let font = include_bytes!("../../fonts/JetBrainsMono-Bold.ttf");
        let builder = BrushBuilder::using_font_bytes(font).unwrap().with_multisample(
            wgpu::MultisampleState {
                count: multisampled_texture.sample_count(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            }
        );
        let brush = builder
         /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(device, surface_configuration.width, surface_configuration.height, wgpu::TextureFormat::Bgra8UnormSrgb);
        Self {
            brush,
            scale_factor,
            std_out: LimitedVec::new(10),
            metrics: Metrics::new(),
            informations: Informations::default(),
            elapsed_time: 0.0,
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.elapsed_time += delta;
        self.metrics.push_data(MetricData::AbsoluteRenderTime(delta));
    }

    pub fn queue(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.elapsed_time < 1000.0 {
            return;
        }
        self.elapsed_time = 0.0;
        let informations_string = self.informations.to_string();
        let metrics_string = self.metrics.to_string();
        let left_informations = format!("{metrics_string}\n---\n{informations_string}");
        let std_out_string = self.std_out.as_vecdeque().iter().fold(String::new(), |mut acc, v| {
            acc.insert_str(acc.len(), &v);
            acc.insert_str(acc.len(), "\n");
            acc
        });
        let size = self.size(14.0);
        let sections = vec![
            
            TextSection::new()
                .add_text(Text::new(&std_out_string).with_scale(size))
                .with_screen_position((self.size(200.0), self.size(10.0))),
                

            TextSection::new()
                .add_text(Text::new(&left_informations).with_scale(size))
                .with_screen_position((self.size(10.0), self.size(10.0))),
                
        ];
        self.brush.queue(device, queue, sections).unwrap();
    }

    fn size(&self, size: f32) -> f32 {
        size * self.scale_factor
    }


}

pub struct Informations {
    pub position: [i32; 3],
    pub cycle_time: (u32, u32)
}

impl Informations {
    pub fn to_string(&self) -> String {
        format!(
            "[{}, {}, {}]\n{}:{}", 
            self.position[0], 
            self.position[1], 
            self.position[2], 
            u8_to_string_with_len(self.cycle_time.0 as u8, 2),
            u8_to_string_with_len(self.cycle_time.1 as u8, 2),
        )
    }
}

impl Default for Informations {
    fn default() -> Self {
        Self { position: [0,0,0], cycle_time: (0, 0) }
    }
}

pub struct Metrics {
    absolute_render_time: Metric,
    logical_render_time: Metric,
    update_call: Metric,
    render_call: Metric,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            absolute_render_time: Metric::new(),
            logical_render_time: Metric::new(),
            update_call: Metric::new(),
            render_call: Metric::new(),
        }
    }
    pub fn to_string(&self) -> String {
        let absolute = calculate_fps(self.absolute_render_time.mean) as u32;
        let logical = calculate_fps(self.logical_render_time.mean) as u32;
        let update = to_fixed_2(self.update_call.mean);
        let render = to_fixed_2(self.render_call.mean);
        format!("{absolute} fps\n{logical} fps w/o submit\n{update} ms update\n{render} ms render")
    }
    pub fn push_data(&mut self, data: MetricData) {
        match data {
            MetricData::AbsoluteRenderTime(v) => self.absolute_render_time.add(v),
            MetricData::LogicalRenderTime(v) => self.logical_render_time.add(v),
            MetricData::UpdateCallTime(v) => self.update_call.add(v),
            MetricData::RenderCallTime(v) => self.render_call.add(v),
        };
    }
}

pub enum MetricData {
    AbsoluteRenderTime(f64),
    LogicalRenderTime(f64),
    RenderCallTime(f64),
    UpdateCallTime(f64),
}

struct Metric {
    data: LimitedVec<f64>,
    pub mean: f64,
}

impl Metric {
    fn new() -> Self { 
        Self { 
            data: LimitedVec::new(10),
            mean: 0.0,
        }
    }
    fn add(&mut self, data: f64) {
        self.data.push(data);
        self.mean = self.data.as_vecdeque().iter().fold(0.0, |acc, v| acc + *v) / self.data.len() as f64
    }
}