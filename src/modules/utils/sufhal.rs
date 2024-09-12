use cgmath::Matrix4;
use wgpu::util::DeviceExt;

use crate::modules::{camera::camera::CameraUniform, core::{light_source::LightSource, model::{CustomMesh, SimpleVertex, TransformUniform, Vertex}}, geometry::plane::Plane};

// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/shadow/shader.wgsl
// https://github.com/gfx-rs/wgpu-rs/blob/master/examples/shadow/main.rs#L750

pub struct ShadowTest {
    texture: wgpu::Texture,
    pub light: LightSource,

    pub pipeline: wgpu::RenderPipeline,
    pub bgl: wgpu::BindGroupLayout,
    pub mesh: CustomMesh,
    pub camera_buffer: wgpu::Buffer,
}

impl ShadowTest {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        multisampled_texture: &wgpu::Texture,

    ) -> Self {

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

        let bgl = Self::get_bgl(device);
        let pipeline = Self::get_pipeline(device, config, multisampled_texture, &bgl);

        let plane = Plane::new(30.0, 30.0, 1, 1);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test Vertex Buffer"),
            contents: bytemuck::cast_slice(&plane.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test Index Buffer"),
            contents: bytemuck::cast_slice(&plane.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test Transform Buffer"),
            contents: bytemuck::cast_slice(&[TransformUniform::from(Matrix4::from_translation([384.0, 178.0, 740.0].into()).into())]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Test Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::new()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            ..Default::default()
        });

        let shadow_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&shadow_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&shadow_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: camera_buffer.as_entire_binding(),
            },
        ];

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &entries,
            label: None,
        });

        let mesh = CustomMesh {
            name: "test shadow".to_string(),
            transform_buffer,
            vertex_buffer,
            index_buffer,
            num_elements: plane.indices.len() as u32,
            bind_group
        };

        Self {
            texture,
            light: LightSource::new([0.0, 0.0, 0.0], view),
            pipeline,
            bgl,
            mesh,
            camera_buffer,
        }
    }

    pub fn get_bgl(
        device: &wgpu::Device
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
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
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                // camera
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("test_bind_group_layout"),
        })
    }

    pub fn get_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        multisampled_texture: &wgpu::Texture,
        bgl: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Test Pipeline Layout"),
            bind_group_layouts: &[
                bgl
            ],
            push_constant_ranges: &[],
        });
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shadow Apply Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/test.wgsl").into()
                ),
            }
        );
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
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
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
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