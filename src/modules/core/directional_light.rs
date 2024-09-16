use cgmath::{perspective, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Transform, Vector3, Vector4, Zero};
use crate::modules::camera::camera::OPENGL_TO_WGPU_MATRIX;

use super::texture::Texture;

pub struct DirectionalLight {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub shadow_texture: Texture,
    cascade_splits: [f32; 4],
    cascade_projections: [Matrix4<f32>; 3],
    pub cascade_textures: [Texture; 3],
}

impl DirectionalLight {

    pub fn new(near: f32, far: f32, position: [f32; 3], target: [f32; 3], device: &wgpu::Device) -> Self {
        let shadow_texture = Self::create_texture(device);
        // PSSM
        let lambda = 0.5;
        let cascade_splits = [
            near,
            calculate_split_dist(near, far, 1, 3, lambda),
            calculate_split_dist(near, far, 2, 3, lambda),
            far,
        ];
        let cascade_projections = [Matrix4::identity(); 3];
        let cascade_textures = [
            Self::create_texture(device),
            Self::create_texture(device),
            Self::create_texture(device),
        ];
        Self {
            position: position.into(),
            target: target.into(),
            shadow_texture,
            cascade_splits,
            cascade_projections,
            cascade_textures,
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

    pub fn update(&mut self, camera_view_proj: Matrix4<f32>) {
        for i in 0..3 {
            let near = self.cascade_splits[i];
            let far = self.cascade_splits[i + 1];

            let corners = get_frustum_corners_in_world_space(camera_view_proj, near, far);
            let view_proj = calculate_light_view_proj(&corners, &self);
            
            self.cascade_projections[i] = view_proj;
        }
    }

    pub fn uniform_from_camera(&self, camera_view_proj: Matrix4<f32>) -> DirectionalLightUniform {
        DirectionalLightUniform {
            view_proj: self.cascade_projections[0].into(),
            ..Default::default()
        }
    }

    pub fn create_texture(device: &wgpu::Device) -> Texture {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Texture"),
            size: wgpu::Extent3d {
                width: 512,
                height: 512,
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
        Texture { view, sampler, texture }
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

fn calculate_split_dist(near: f32, far: f32, i: u32, splits: u32, lambda: f32) -> f32 {
    let uniform = near + (far - near) * (i as f32 / splits as f32);
    let logarithmic = near * (far / near).powf(i as f32 / splits as f32);
    lambda * uniform + (1.0 - lambda) * logarithmic
}

fn calculate_light_view_proj(frustum_corners: &[Point3<f32>; 8], directional_light: &DirectionalLight) -> Matrix4<f32> {
    let light_direction = Vector3::new(-0.5, -1.0, -0.5).normalize(); // Diagonale depuis la gauche et vers l'arrière
    let frustum_center = get_frustum_center(&frustum_corners);

    let light_distance = 100.0;
    let light_position = frustum_center - light_direction * light_distance;

    let light_view = Matrix4::look_at_rh(
        light_position,
        frustum_center,
        Vector3::unit_y()
    );

    let (min, max) = oriented_bounding_box(frustum_corners, &light_view);

    let margin = 10.0;
    let light_projection = OPENGL_TO_WGPU_MATRIX * cgmath::ortho(
        min.x - margin, max.x + margin,
        min.y - margin, max.y + margin,
        min.z - margin, max.z + margin
    );

    light_projection * light_view
}


fn oriented_bounding_box(corners: &[Point3<f32>], light_view: &Matrix4<f32>) -> (Point3<f32>, Point3<f32>) {
    let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);

    for corner in corners {
        let transformed_corner = light_view.transform_point(*corner);
        min = min.zip(transformed_corner, |a, b| a.min(b));
        max = max.zip(transformed_corner, |a, b| a.max(b));
    }

    (min, max)
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

fn get_frustum_corners_in_world_space(camera_view_proj: Matrix4<f32>, near: f32, far: f32) -> [Point3<f32>; 8] {
    let inv_camera_view_proj = camera_view_proj.invert().unwrap();
    let mut corners = [Point3::new(0.0, 0.0, 0.0); 8];

    for i in 0..8 {
        let x = if i & 1 == 0 { -1.0 } else { 1.0 };
        let y = if i & 2 == 0 { -1.0 } else { 1.0 };
        let z = if i & 4 == 0 { near } else { far };

        let point = inv_camera_view_proj * Vector4::new(x, y, z, 1.0);
        corners[i] = Point3::new(point.x / point.w, point.y / point.w, point.z / point.w);
    }

    corners
}


fn get_frustum_center(frustum_corners: &[Point3<f32>; 8]) -> Point3<f32> {
    // Prendre les points proches (0-3) et éloignés (4-7)
    let mut center = Point3::new(0.0, 0.0, 0.0);

    for corner in frustum_corners {
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