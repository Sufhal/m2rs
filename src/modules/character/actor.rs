use std::f32::consts::{FRAC_2_PI, PI};

use cgmath::{InnerSpace, Matrix3, Matrix4, Rad, Transform, Vector3};
use winit::{event::ElementState, keyboard::KeyCode};
use crate::modules::{camera::{camera::Camera, orbit_controller::OrbitController}, core::scene::Scene};
use super::character::{Character, CharacterState};


pub struct Actor {
    pub character: String,
    pub orbit_controller: OrbitController,
    controller: Controller,
}

impl Actor {

    pub fn new(character: String) -> Self {
        Self {
            character,
            controller: Default::default(),
            orbit_controller: OrbitController::new(),
        }
    }

    pub fn apply_controls(&self, character: &mut Character, scene: &mut Scene, camera: &Camera, delta_ms: f32) {
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
            let rotation = Matrix4::from_angle_y(-camera.yaw - Rad(PI / 2.0));
            let camera_space_movement = rotation.transform_vector(movement_direction);
            let movement = Vector3::new(camera_space_movement.x, 0.0, camera_space_movement.z).normalize();
            character.move_in_direction(movement, scene, delta_ms);
            character.set_state(CharacterState::Run, scene);
        } else {
            character.set_state(CharacterState::Wait, scene);
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.controller.backward = false;
                self.controller.forward = state.is_pressed();
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.controller.forward = false;
                self.controller.backward = state.is_pressed();
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.controller.right = false;
                self.controller.left = state.is_pressed();
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.controller.left = false;
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