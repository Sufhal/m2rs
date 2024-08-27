use crate::modules::core::model::Mesh;

pub trait ToMesh {
    fn to_mesh(&self, device: &wgpu::Device, name: String) -> Mesh;
}

pub trait ToSkinnedMesh {
    fn to_skinned_mesh(&self, device: &wgpu::Device, name: String) -> Mesh;
}