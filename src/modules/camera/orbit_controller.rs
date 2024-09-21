use cgmath::{num_traits::Signed, Point3, Rad};
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta};
use crate::modules::{core::object_3d::GroundAttachable, terrain::terrain::Terrain};

use super::camera::Camera;

const VERTICAL_OFFSET: f32 = 2.0;

pub struct OrbitController {
    target: Point3<f32>,
    distance: f32,
    yaw: f32,
    pitch: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
}

impl OrbitController {

    pub fn new() -> Self {
        Self {
            target: Point3::new(0.0, 0.0, 0.0),
            distance: 10.0,
            yaw: 0.0,
            pitch: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
        }
    }

    pub fn process_mouse(&mut self, delta_x: f64, delta_y: f64) {
        self.rotate_horizontal = delta_x as f32;
        self.rotate_vertical = delta_y as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        let value = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
        self.distance += value / 15.0;
        self.distance = self.distance.clamp(2.0, 30.0);
        dbg!(self.distance);
    }

    pub fn update_target(&mut self, mut target: [f32; 3]) {
        target[1] += VERTICAL_OFFSET;
        self.target = Point3::from(target);
    }

    pub fn update_camera(&mut self, camera: &mut Camera, terrain: &Terrain) {
        const MINIMUM_HEIGHT: f32 = 1.0;
        let distance_to_ground_bottom = camera.get_distance_to_ground([0.0, -1.0, 0.0], terrain);
        self.yaw += self.rotate_horizontal * 0.01;
        if distance_to_ground_bottom > MINIMUM_HEIGHT || self.rotate_vertical.is_positive() {
            self.pitch += self.rotate_vertical * 0.01;
        }
        const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.1;
        self.pitch = self.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let horizontal_distance = self.distance * self.pitch.cos();
        let x = horizontal_distance * self.yaw.cos();
        let z = horizontal_distance * self.yaw.sin();
        let y = self.distance * self.pitch.sin();
        camera.position = Point3::new(
            self.target.x + x,
            self.target.y + y,
            self.target.z + z,
        );
        camera.yaw = Rad(self.yaw + std::f32::consts::PI);
        camera.pitch = Rad(-self.pitch);
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
    }

}