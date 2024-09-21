use std::f32::consts::PI;
use cgmath::{Angle, InnerSpace, Matrix4, Rad, Transform, Vector3};
use winit::{event::ElementState, keyboard::KeyCode};
use crate::modules::{camera::{camera::Camera, orbit_controller::OrbitController}, core::scene::Scene, utils::functions::lerp_angle};
use super::character::{Character, CharacterState};

const ROTATION_SPEED: f32 = PI / (150.0 / 1000.0);
//                                ^^^^^ duration in ms to make a a 360

pub struct Actor {
    pub character: String,
    pub orbit_controller: OrbitController,
    controller: Controller,
    direction: Vector3<f32>,
    rotation_speed: f32,
}

impl Actor {

    pub fn new(character: String) -> Self {
        Self {
            character,
            controller: Default::default(),
            orbit_controller: OrbitController::new(),
            direction: Vector3::new(0.0, 0.0, 0.0), 
            rotation_speed: ROTATION_SPEED,
        }
    }

    pub fn apply_controls(&mut self, character: &mut Character, scene: &mut Scene, camera: &Camera, delta_ms: f32) {
        let mut movement_direction = Vector3::new(0.0, 0.0, 0.0);
        if self.controller.forward {
            movement_direction.z -= 1.0;
        }
        if self.controller.backward {
            movement_direction.z += 1.0;
        }
        if self.controller.left {
            movement_direction.x -= 1.0;
        }
        if self.controller.right {
            movement_direction.x += 1.0;
        }

        if self.controller.attack == true {
            character.set_state(CharacterState::Attack, scene);
        } else if
            self.controller.forward == true || 
            self.controller.backward == true || 
            self.controller.left == true || 
            self.controller.right == true  
        {
            if movement_direction.magnitude2() < 0.0001 { // trying to normalize a zero vector result in a NaN vector
                character.set_state(CharacterState::Wait, scene);
                return
            }
            let rotation = Matrix4::from_angle_y(-camera.yaw - Rad(PI / 2.0));
            let camera_space_movement = rotation.transform_vector(movement_direction);
            let desired_direction = Vector3::new(camera_space_movement.x, 0.0, camera_space_movement.z).normalize();
            let delta_seconds = delta_ms / 1000.0;
            let current_angle = Rad::atan2(self.direction.z, self.direction.x);
            let desired_angle = Rad::atan2(desired_direction.z, desired_direction.x);
            let new_angle = lerp_angle(current_angle, desired_angle, self.rotation_speed * delta_seconds);
            self.direction = Vector3::new(new_angle.cos(), 0.0, new_angle.sin()).normalize();
            character.move_in_direction(self.direction, scene, delta_ms);
            character.set_state(CharacterState::Run, scene);
        } else {
            character.set_state(CharacterState::Wait, scene);
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.controller.forward = state.is_pressed();
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.controller.backward = state.is_pressed();
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.controller.left = state.is_pressed();
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.controller.right = state.is_pressed();
                true
            }
            KeyCode::Space => {
                self.controller.attack = state.is_pressed();
                true
            },
            _ => false
        }
    }

}


#[derive(Debug)]
struct Controller {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    attack: bool,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            attack: false,
        }
    }
}