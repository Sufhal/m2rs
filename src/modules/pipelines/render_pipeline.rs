use wgpu::util::DeviceExt;

use crate::modules::{camera::camera::CameraUniform, core::{instance::InstanceRaw, light::LightUniform, model::{ModelVertex, Vertex}}};

pub struct RenderBindGroupLayouts {
    pub global: wgpu::BindGroupLayout,
    pub mesh: wgpu::BindGroupLayout,
    pub instances: wgpu::BindGroupLayout,
}

pub struct Buffers {
    pub light: wgpu::Buffer,
    pub camera: wgpu::Buffer,
}

pub struct Uniforms {
    pub light: LightUniform,
    pub camera: CameraUniform
}

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub bind_group_layouts: RenderBindGroupLayouts,
    pub global_bind_group: wgpu::BindGroup,
    pub buffers: Buffers,
    pub uniforms: Uniforms,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> Self {
        let global = Self::create_global_layout(device);
        let mesh = Self::create_mesh_layout(device);
        let instances = Self::create_instances_layout(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &global,
                &mesh,
                &instances
            ],
            push_constant_ranges: &[],
        });
        let pipeline = Self::create_pipeline(device, &pipeline_layout, config, depth_format);
        let uniforms = Self::create_uniforms();
        let buffers = Self::create_buffers(device, &uniforms);
        let global_bind_group = Self::create_global_bind_group(device, &global, &buffers);
        Self {
            pipeline,
            pipeline_layout,
            bind_group_layouts: RenderBindGroupLayouts { global, mesh, instances },
            global_bind_group,
            uniforms,
            buffers
        }
    }

    fn create_global_bind_group(device: &wgpu::Device, global_layout: &wgpu::BindGroupLayout, buffers: &Buffers) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.camera.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.light.as_entire_binding(),
                }
            ],
            label: Some("light_bind_group"),
        })
    }

    fn create_uniforms() -> Uniforms {
        Uniforms {
            light: LightUniform {
                position: [2.0, 2.0, 2.0],
                _padding: 0,
                color: [1.0, 1.0, 1.0],
                _padding2: 0,
            },
            camera: CameraUniform::new()
        }
    }

    fn create_buffers(device: &wgpu::Device, uniforms: &Uniforms) -> Buffers {
        Buffers {
            light: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[uniforms.light]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            camera: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniforms.camera]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    fn create_global_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // camera
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
                // lights
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("global_bind_group_layout"),
        })
    }

    fn create_instances_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // skeletons matrices
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // skeletons bind inverse matrices
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // skinning informations
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },  
            ],
            label: Some("instances_bind_group_layout"),
        })
    }

    fn create_mesh_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // transform
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("mesh_bind_group_layout"),
        })
    }

    fn create_pipeline(
        device: &wgpu::Device, 
        layout: &wgpu::PipelineLayout,
        config: &wgpu::SurfaceConfiguration,
        depth_format: Option<wgpu::TextureFormat>,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/shader_v2.wgsl").into()
                ),
            }
        );
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("New Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    ModelVertex::desc(), 
                    InstanceRaw::desc()
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
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    } 
}