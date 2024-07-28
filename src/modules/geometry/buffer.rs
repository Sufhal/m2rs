use wgpu::util::DeviceExt;
use crate::modules::core::model::{Mesh, ModelVertex};

pub trait ToMesh {
    fn to_mesh(&self, device: &wgpu::Device, transform_bind_group_layout: &wgpu::BindGroupLayout, name: String) -> Mesh;
}