use wgpu::util::DeviceExt;
use crate::modules::core::model::{Mesh, SkinnedVertex};

pub trait ToMesh {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh;
}