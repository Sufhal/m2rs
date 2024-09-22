use cgmath::{Deg, Matrix4, Rad, Vector3};

use crate::modules::{core::{object, object_3d::Object3D, scene::Scene, skinning::SkeletonInstance}, state::State, utils::functions::decompose_matrix};

use super::character::Character;

pub enum Weapon {
    None,
    Single(String, String),
    Dual([(String, String); 2]),
}

impl Weapon {
    pub fn update(&self, character: &Character, scene: &mut Scene) {
        match self {
            Self::Single(object_id, instance_id) => {
                if let Some(equip_right_matrix) = character.get_equip_right_matrix(&scene) {
                    let character_matrix = character.get_matrix(&scene);
                    let world_matrix = character_matrix * equip_right_matrix * Matrix4::from_angle_z(Deg(-45.0));
                    if let Some(object) = scene.get_mut(object_id) {
                        if let Some(object3d) = &mut object.object3d {
                            match object3d {
                                Object3D::Simple(simple) => {
                                    if let Some(instance) = simple.get_instance(&instance_id) {
                                        let (translation, rotation, scale) = decompose_matrix(&world_matrix);
                                        instance.set_position(translation);
                                        instance.set_rotation(rotation);
                                        // instance.set_scale(scale);
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