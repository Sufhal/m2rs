use cgmath::{Matrix4, Vector3};

use crate::modules::{core::{object, object_3d::Object3D, scene::Scene, skinning::SkeletonInstance}, state::State};

use super::character::Character;

pub enum Weapon {
    None,
    Single(String, String),
    Dual([(String, String); 2]),
}

impl Weapon {
    pub fn update(&self, character: &Character, scene: &mut Scene) {
        println!("weapon update");
        match self {
            Self::Single(object_id, instance_id) => {
                if let Some(matrix_world) = character.get_equip_right_matrix(&scene) {
                    if let Some(object) = scene.get_mut(object_id) {
                        if let Some(object3d) = &mut object.object3d {
                            match object3d {
                                Object3D::Simple(simple) => {
                                    if let Some(instance) = simple.get_instance(&instance_id) {
                                        let position = Vector3::from([
                                            matrix_world.w.x,
                                            matrix_world.w.y,
                                            matrix_world.w.z,
                                        ]);
                                        dbg!(&position);
                                        instance.set_position(position);
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                }
            },
            _ => (),
        }
    }
}