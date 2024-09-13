use std::{default, ops::Range};
use cgmath::{perspective, Deg, EuclideanSpace, InnerSpace, Matrix4, PerspectiveFov, Point3, Vector3};
use cgmath::*;

use crate::modules::camera::camera::{Projection, OPENGL_TO_WGPU_MATRIX};

pub struct LightSource {
    pub position: Point3<f32>,
    // projection: Projection,
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

    pub fn uniform(&self, aspect: f32) -> LightSourceUniform {

        let light_position = Point3::new(-26.0, 45.0, 61.0);
        let light_target = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        // let up = Vector3::unit_y();

            // Calculer la matrice de vue (view matrix)
        let light_view = Matrix4::look_at_rh(light_position, light_target, up);

        // Calculer la matrice de projection
        let light_projection = OPENGL_TO_WGPU_MATRIX * perspective(Deg(28.0), aspect, 0.1, 100.0);

        // Combiner les matrices de vue et de projection
        let mx_view_proj = light_projection * light_view;
        LightSourceUniform {
            view_position: light_position.to_homogeneous().into(),
            view_proj: mx_view_proj.into(),
            view_matrix: light_view.into(),
            projection_matrix: light_projection.into(),
        }
    }


}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSourceUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
}

impl Default for LightSourceUniform {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            view_proj: Default::default(),
            view_matrix: Default::default(),
            projection_matrix: Default::default(),
        }
    }
}

// impl LightSourceUniform {
//     pub fn update(&mut self, light_source: &LightSource) {
//         let projection_matrix = light_source.projection.calc_matrix();
//         let view_matrix = camera.calc_matrix();
//         self.view_projection = (projection_matrix * view_matrix).into();
//     }
// }