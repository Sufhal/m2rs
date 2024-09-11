use crate::modules::core::{instance::InstanceRaw, model::{SimpleVertex, Vertex}};
use super::{common_pipeline::CommonPipeline, simple_models_pipeline::SimpleModelPipeline};

pub struct ShadowPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
}

impl ShadowPipeline {
    pub fn new(
        device: &wgpu::Device,
        common_pipeline: &CommonPipeline,
        simple_models_pipeline: &SimpleModelPipeline,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[
                &simple_models_pipeline.bind_group_layouts.mesh
            ],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(device, &pipeline_layout);
        Self {
            pipeline,
            pipeline_layout
        }
    }

    fn create_pipeline(
        device: &wgpu::Device, 
        layout: &wgpu::PipelineLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/shadow.wgsl").into()
                ),
            }
        );
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    SimpleVertex::desc(), 
                    InstanceRaw::desc()
                ],
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    } 
}