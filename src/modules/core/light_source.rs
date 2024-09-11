use std::{default, ops::Range};
use cgmath::{Deg, EuclideanSpace, Matrix4, PerspectiveFov, Point3, Vector3};

use crate::modules::camera::camera::OPENGL_TO_WGPU_MATRIX;

pub struct LightSource {
    position: Point3<f32>,
    fov: f32,
    depth: Range<f32>,
    pub target_view: wgpu::TextureView,
}

impl LightSource {

    pub fn new(position: [f32; 3], target_view: wgpu::TextureView) -> Self {
        Self {
            position: position.into(),
            fov: 28.0,
            depth: 0.1..100.0,
            target_view
        }
    }

    pub fn uniform(&self) -> LightSourceUniform {
        let mx_view = Matrix4::look_at_rh(self.position, Point3::origin(), Vector3::unit_z());
        let projection = PerspectiveFov {
            fovy: Deg(self.fov).into(),
            aspect: 1.0,
            near: self.depth.start,
            far: self.depth.end,
        };
        let mx_view_proj =
            OPENGL_TO_WGPU_MATRIX * cgmath::Matrix4::from(projection.to_perspective()) * mx_view;
        LightSourceUniform {
            view_projection: *mx_view_proj.as_ref()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSourceUniform {
    pub view_projection: [[f32; 4]; 4],
}

impl Default for LightSourceUniform {
    fn default() -> Self {
        Self {
            view_projection: Default::default()
        }
    }
}