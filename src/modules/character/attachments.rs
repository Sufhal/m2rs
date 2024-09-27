use crate::modules::{core::{object_3d::Object3D, scene::Scene}, utils::functions::decompose_matrix};
use super::character::Character;

pub struct Attachments {
    pub hair: Option<Hair>,
    pub weapon: Weapon,
}

impl Attachments {
    pub async fn new() -> Self {
        Self {
            hair: None,
            weapon: Weapon::None,
        }
    }
}

pub enum AttachmentType {
    Weapon,
    Hair,
}

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
                    let world_matrix = character_matrix * equip_right_matrix;
                    // let world_matrix = character_matrix * equip_right_matrix * Matrix4::from_angle_z(Deg(-45.0));
                    if let Some(object) = scene.get_mut(object_id) {
                        if let Some(object3d) = &mut object.object3d {
                            match object3d {
                                Object3D::Simple(simple) => {
                                    if let Some(instance) = simple.get_instance(&instance_id) {
                                        let (translation, rotation, _scale) = decompose_matrix(&world_matrix);
                                        instance.set_position(translation);
                                        instance.set_rotation(rotation);
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

pub struct Hair(pub String, pub String);

impl Hair {
    pub fn update(&self, character: &Character, scene: &mut Scene, queue: &wgpu::Queue) {
        let object_id = &self.0;
        let instance_id = &self.1;
        // let mixer = character.get_mixer(&scene).unwrap();
        let skeleton = character.get_skeleton_instance(&scene).unwrap();
        let character_matrix = character.get_matrix(&scene);
        let world_matrix = character_matrix;
        if let Some(object) = scene.get_mut(object_id) {
            if let Some(object3d) = &mut object.object3d {
                match object3d {
                    Object3D::Skinned(skinned) => {
                        if let Some(instance) = skinned.get_instance(&instance_id) {
                            // instance.mixer = mixer;
                            instance.skeleton = skeleton.clone();
                            let (translation, rotation, _scale) = decompose_matrix(&world_matrix);
                            instance.set_position(translation);
                            instance.set_rotation(rotation);
                        }
                        skinned.update_one_skeleton(instance_id, queue);
                    },
                    _ => (),
                }
            }
        }
    }
}