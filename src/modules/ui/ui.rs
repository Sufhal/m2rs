use std::ops::{Add, Div};

use wgpu_text::{glyph_brush::{ab_glyph::FontRef, Section as TextSection, Text}, BrushBuilder, TextBrush};

use crate::modules::utils::{functions::{calculate_fps, to_fixed_2}, structs::LimitedVec, time_factory::TimeFactory};

pub struct UserInterface {
    pub brush: TextBrush<FontRef<'static>>,
    pub metrics: Metrics,
    scale_factor: f32,
    elapsed_time: f64,
}

impl UserInterface {

    pub fn new(device: &wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration, scale_factor: f32) -> Self {
        let font = include_bytes!("../../fonts/JetBrainsMono-Bold.ttf");
        let brush = BrushBuilder::using_font_bytes(font).unwrap()
         /* .initial_cache_size((16_384, 16_384))) */ // use this to avoid resizing cache texture
            .build(device, surface_configuration.width, surface_configuration.height, wgpu::TextureFormat::Bgra8UnormSrgb);
        Self {
            brush,
            scale_factor,
            metrics: Metrics::new(),
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
        let metrics_string = self.metrics.to_string();
        let size = self.size(14.0);
        let sections = vec![
            TextSection::new()
                .add_text(Text::new(&metrics_string).with_scale(size))
                .with_screen_position((self.size(10.0), self.size(10.0)))
        ];
        self.brush.queue(device, queue, sections).unwrap();
    }

    fn size(&self, size: f32) -> f32 {
        size * self.scale_factor
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
        format!("{absolute} fps\n{logical} fps w/ submit\n{update} ms update\n{render} ms render")
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
        self.mean = self.data.as_vecdeque().iter().fold(0.0, |mut acc, v| acc + *v) / self.data.len() as f64
    }
}