use cgmath::{perspective, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4, Zero};
use crate::modules::camera::camera::OPENGL_TO_WGPU_MATRIX;

use super::texture::Texture;

pub struct DirectionalLight {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub shadow_texture: Texture,
}

impl DirectionalLight {

    pub fn new(position: [f32; 3], target: [f32; 3], device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: 4096,
                height: 4096,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[]
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = Texture { view, sampler, texture };
        
        Self {
            position: position.into(),
            target: target.into(),
            shadow_texture,
        }
    }

    pub fn uniform(&self, aspect: f32) -> DirectionalLightUniform {
        let light_view = Matrix4::look_at_rh(self.position, self.target, Vector3::unit_y());
        let light_projection = OPENGL_TO_WGPU_MATRIX * perspective(Deg(28.0), aspect, 0.1, 100.0);
        let mx_view_proj = light_projection * light_view;

        DirectionalLightUniform {
            view_position: self.position.to_homogeneous().into(),
            view_proj: mx_view_proj.into(),
            view_matrix: light_view.into(),
            projection_matrix: light_projection.into(),
        }
    }

    pub fn uniform_from_camera(&self, camera_view_proj: Matrix4<f32>) -> DirectionalLightUniform {
        DirectionalLightUniform {
            view_proj: calculate_light_view_proj(camera_view_proj, self).into(),
            ..Default::default()
        }
    }


}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightUniform {
    pub view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    pub view_matrix: [[f32; 4]; 4],
    pub projection_matrix: [[f32; 4]; 4],
}

impl Default for DirectionalLightUniform {
    fn default() -> Self {
        Self {
            view_position: Default::default(),
            view_proj: Default::default(),
            view_matrix: Default::default(),
            projection_matrix: Default::default(),
        }
    }
}

fn calculate_light_view_proj(camera_view_proj: Matrix4<f32>, directional_light: &DirectionalLight) -> Matrix4<f32> {

    let light_direction = Vector3::new(-0.5, -1.0, -0.5).normalize(); // Diagonale depuis la gauche et vers l'arrière
    let frustum_center = get_frustum_center(camera_view_proj);
    let light_position = frustum_center - light_direction * 100.0;

    let light_view = Matrix4::look_at_rh(
        light_position,
        frustum_center,
        Vector3::unit_y()
    );

    let light_projection = OPENGL_TO_WGPU_MATRIX * perspective(Deg(28.0), 1.0, 0.1, 100.0);

    light_projection * light_view
}


fn get_frustum_corners(camera_view_proj: Matrix4<f32>) -> [Point3<f32>; 8] {
    let inverse_view_proj = camera_view_proj.invert().unwrap();
    let mut frustum_corners = [
        Vector4::new(-1.0, -1.0, -1.0, 1.0),  // Near bottom left
        Vector4::new(1.0, -1.0, -1.0, 1.0),   // Near bottom right
        Vector4::new(1.0, 1.0, -1.0, 1.0),    // Near top right
        Vector4::new(-1.0, 1.0, -1.0, 1.0),   // Near top left
        Vector4::new(-1.0, -1.0, 1.0, 1.0),   // Far bottom left
        Vector4::new(1.0, -1.0, 1.0, 1.0),    // Far bottom right
        Vector4::new(1.0, 1.0, 1.0, 1.0),     // Far top right
        Vector4::new(-1.0, 1.0, 1.0, 1.0),    // Far top left
    ];

    for corner in &mut frustum_corners {
        *corner = inverse_view_proj * *corner;
        *corner /= corner.w;
    }

    let frustum_points = frustum_corners.map(|v| Point3::new(v.x, v.y, v.z));
    frustum_points
}

fn get_frustum_center(camera_view_proj: Matrix4<f32>) -> Point3<f32> {
    let frustum_corners = get_frustum_corners(camera_view_proj);

    // Prendre les points proches (0-3) et éloignés (4-7)
    let mut center = Point3::new(0.0, 0.0, 0.0);

    for corner in &frustum_corners {
        center.x += corner.x;
        center.y += corner.y;
        center.z += corner.z;
    }

    // Diviser par 8 pour obtenir la moyenne
    center.x /= 8.0;
    center.y /= 8.0;
    center.z /= 8.0;

    center
}