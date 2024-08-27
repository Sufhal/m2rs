use wgpu::util::DeviceExt;
use crate::modules::{camera::camera::CameraUniform, core::light::LightUniform};

pub struct Buffers {
    pub light: wgpu::Buffer,
    pub camera: wgpu::Buffer,
}

pub struct Uniforms {
    pub light: LightUniform,
    pub camera: CameraUniform
}

pub struct CommonPipeline {
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
    pub buffers: Buffers,
    pub uniforms: Uniforms
}

impl CommonPipeline {

    pub fn new(
        device: &wgpu::Device,
    ) -> Self {
        let uniforms = Self::create_uniforms();
        let buffers = Self::create_buffers(device, &uniforms);
        let global_bind_group_layout = Self::create_global_bind_group_layout(device);
        let global_bind_group = Self::create_global_bind_group(device, &global_bind_group_layout, &buffers);
        Self {
            global_bind_group_layout,
            global_bind_group,
            buffers,
            uniforms,
        }
    }

    fn create_global_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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

}