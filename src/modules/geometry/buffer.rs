use crate::modules::core::model::Mesh;

pub trait ToMesh {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh;
}