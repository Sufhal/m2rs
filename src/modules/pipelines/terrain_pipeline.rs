use crate::modules::core::model::{SimpleVertex, Vertex};
use super::common_pipeline::CommonPipeline;

pub struct TerrainBindGroupLayouts {
    pub mesh: wgpu::BindGroupLayout,
}

pub struct TerrainPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bind_group_layouts: TerrainBindGroupLayouts,
}

impl TerrainPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        depth_format: Option<wgpu::TextureFormat>,
        multisampled_texture: &wgpu::Texture,
        common_pipeline: &CommonPipeline
    ) -> Self {
        let mesh = Self::create_mesh_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain Pipeline Layout"),
            bind_group_layouts: &[
                &common_pipeline.global_bind_group_layout,
                &mesh,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(device, &pipeline_layout, config, depth_format, multisampled_texture);
        Self {
            pipeline,
            pipeline_layout,
            bind_group_layouts: TerrainBindGroupLayouts { mesh },
        }
    }

    fn create_mesh_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let mut entries = vec![
            // transform
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // chunk informations
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // sampler (texture)
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ];

        for i in 0..8 {
            let offset = 3;
            // texture
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: offset + (i) as u32,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
        }

        // alpha atlas
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        });

        entries.push(wgpu::BindGroupLayoutEntry {
            binding: entries.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("terrain_mesh_bind_group_layout"),
        })
    }

    fn create_pipeline(
        device: &wgpu::Device, 
        layout: &wgpu::PipelineLayout,
        config: &wgpu::SurfaceConfiguration,
        depth_format: Option<wgpu::TextureFormat>,
        multisampled_texture: &wgpu::Texture,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Terrain Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/terrain.wgsl").into()
                ),
            }
        );
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    SimpleVertex::desc(), 
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format.add_srgb_suffix(),
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
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
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: multisampled_texture.sample_count(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    } 
}