use wgpu::util::DeviceExt;

use crate::modules::core::{instance::InstanceRaw, light_source::LightSourceUniform, model::{SimpleVertex, Vertex}};
use super::{common_pipeline::CommonPipeline, simple_models_pipeline::SimpleModelPipeline};

pub struct Buffers {
    pub light_source: wgpu::Buffer,
}

pub struct Uniforms {
    pub light_source: LightSourceUniform,
}

pub struct ShadowPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub buffers: Buffers,
    pub uniforms: Uniforms
}

impl ShadowPipeline {
    pub fn new(
        device: &wgpu::Device,
        simple_models_pipeline: &SimpleModelPipeline,
    ) -> Self {
        let uniforms = Self::create_uniforms();
        let buffers = Self::create_buffers(device, &uniforms);
        let bind_group_layout = Self::create_bind_group_layout(device);
        let bind_group = Self::create_bind_group(device, &bind_group_layout, &buffers);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
                &simple_models_pipeline.bind_group_layouts.mesh
            ],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(device, &pipeline_layout);
        Self {
            pipeline,
            pipeline_layout,
            bind_group,
            bind_group_layout,
            uniforms,
            buffers,
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
                unclipped_depth: true,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
                
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // bias: wgpu::DepthBiasState {
                //     constant: 2, // corresponds to bilinear filtering
                //     slope_scale: 2.0,
                //     clamp: 0.0,
                // },
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("light_source_bind_group_layout"),
        })
    }

    fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffers: &Buffers) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.light_source.as_entire_binding(),
                },
            ],
            label: Some("light_source_bind_group"),
        })
    }

    fn create_uniforms() -> Uniforms {
        Uniforms {
            light_source: LightSourceUniform::default()
        }
    }

    fn create_buffers(device: &wgpu::Device, uniforms: &Uniforms) -> Buffers {
        Buffers {
            light_source: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light Source Buffer"),
                contents: bytemuck::cast_slice(&[uniforms.light_source]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

}