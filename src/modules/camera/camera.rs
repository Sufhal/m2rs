use cgmath::*;

use crate::modules::core::{directional_light::DirectionalLight, object_3d::{GroundAttachable, Position, Translate}};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


#[derive(Debug)]
pub struct Camera {
    pub position: Point3<f32>,
    pub use_directional_light: bool,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Rad<f32>>,
        P: Into<Rad<f32>>,
    >(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            use_directional_light: false,
            yaw: yaw.into(),
            pitch: pitch.into(),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            Vector3::unit_y(),
        )
    }
}

impl Position for Camera {
    fn get_position(&mut self) -> [f32; 3] {
        self.position.into()
    }
}

impl Translate for Camera {
    fn translate(&mut self, value: &[f32; 3]) {
        self.position.x = value[0];
        self.position.y = value[1];
        self.position.z = value[2];
    }
}

impl GroundAttachable for Camera {}

pub struct Projection {
    pub aspect: f32,
    fovy: Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
        // OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
            view_matrix: cgmath::Matrix4::identity().into(),
            projection_matrix: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection, directional_light: &DirectionalLight) {
        let projection_matrix = projection.calc_matrix();
        let view_matrix = camera.calc_matrix();
        match camera.use_directional_light {
            true => {
                let uniform = directional_light.uniform_from_camera((projection_matrix * view_matrix).into());
                self.view_position = uniform.view_position;
                self.view_proj = uniform.view_proj;
                self.view_matrix = uniform.view_matrix;
                self.projection_matrix = uniform.projection_matrix;
            }
            false => {
                self.view_position = camera.position.to_homogeneous().into();
                self.view_proj = (projection_matrix * view_matrix).into();
                self.view_matrix = view_matrix.into();
                self.projection_matrix = projection_matrix.into();
            }
        } 
        
    }
}