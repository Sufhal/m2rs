
use std::borrow::Borrow;
use std::fmt::{self};
use std::rc::Rc;
use cgmath::{InnerSpace, Matrix4, Quaternion, Vector3};
use crate::modules::assets::gltf_loader::{load_animation, load_model_glb};
use crate::modules::core::motions::MotionsGroups;
use crate::modules::core::object::Object;
use crate::modules::core::object_3d::{AdditiveTranslation, AdditiveTranslationWithScene, GroundAttachable, Object3D, Rotate, RotateWithScene, Translate, TranslateWithScene};
use crate::modules::core::scene::Scene;
use crate::modules::core::skinning::{AnimationMixer, Skeleton, SkeletonInstance};
use crate::modules::state::State;
use crate::modules::terrain::terrain::Terrain;
use crate::modules::utils::functions::clone_from_rc;
use crate::modules::utils::id_gen::generate_unique_string;
use super::actor::Actor;
use super::attachments::{AttachmentType, Attachments, Hair, Weapon};

pub struct Character {
    pub id: String,
    #[allow(dead_code)]
    kind: CharacterKind,
    mode: CharacterMode,
    pub objects: Vec<(String, String)>, // (Object ID, Object3DInstance ID)
    pub position: [f32; 3],
    #[allow(dead_code)]
    direction: [f32; 3],
    velocity: f32,
    motions: MotionsGroups,
    state: CharacterState,
    pub attachments: Attachments,
    has_moved: bool,
}

impl Character {
    pub async fn new<'a>(name: &str, kind: CharacterKind, state: &mut State<'a>) -> Self {
        Self::load(name, kind, state).await
    }
    async fn load<'a>(name: &str, kind: CharacterKind, state: &mut State<'a>) -> Self {

        let filename = match &kind {
            CharacterKind::NPC(npc) => {
                let npc_type = npc.to_string();
                format!("pack/{npc_type}/{name}/{name}.glb")
            },
            CharacterKind::PC(pc) => {
                let pc_type = pc.to_string();
                format!("pack/pc/{pc_type}/{name}.glb")
            }
        };

        // loading motions descriptions
        let motions = match &kind {
            CharacterKind::NPC(npc) => {
                match npc {
                    NPCType::Monster => {
                        let npc_type = npc.to_string();
                        let filename = format!("pack/{npc_type}/{name}/motions.json");
                        MotionsGroups::load(&filename).await.unwrap()
                    },
                    NPCType::Normal => todo!()
                }
            },
            CharacterKind::PC(pc) => {
                match pc {
                    PCType::Shaman(sex) => {
                        match sex {
                            Sex::Male => {
                                let filename = format!("pack/pc/{}/motions.json", pc.to_string());
                                MotionsGroups::load(&filename).await.unwrap()
                            },
                            Sex::Female => todo!()
                        }
                    },
                    PCType::Warrior(sex) => {
                        match sex {
                            Sex::Male => {
                                let filename = format!("pack/pc/{}/motions.json", pc.to_string());
                                MotionsGroups::load(&filename).await.unwrap()
                            },
                            Sex::Female => todo!()
                        }
                    },
                    _ => todo!()
                }
            }
        };

        let objects = state.scene.create_instance_of(
            &filename,
            &state.device,
            &state.queue,
            &state.skinned_models_pipeline,
            &state.simple_models_pipeline,
            None,
        ).await;

        for (object_id, _instance_id) in &objects {
            if let Some(object) = state.scene.get_mut(object_id) {
                object.matrix = Matrix4::from_scale(1.0).into(); // <- TODO: I should not do that
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Skinned(skinned) => {
                            for group in &motions.groups {
                                for motion in &group.motions {
                                    let animations_path = match &kind {
                                        CharacterKind::NPC(npc) => {
                                            match npc {
                                                NPCType::Monster => {
                                                    let npc_type = npc.to_string();
                                                    format!("pack/{npc_type}/{name}")
                                                },
                                                NPCType::Normal => todo!()
                                            }
                                        },
                                        CharacterKind::PC(pc) => {
                                            let character_path = pc.to_string();
                                            format!("pack/pc/{character_path}")
                                        }
                                    };
                                    let name = &motion.file;
                                    let path = format!("{animations_path}/{name}.glb");
                                    dbg!(&path, &name);
                                    let clip = load_animation(&path, name).await.unwrap();
                                    skinned.add_animation(clip);
                                }
                            }
                        },
                        _ => (),
                    }
                }
            }
        }

        Self {
            id: generate_unique_string(),
            kind,
            objects,
            motions,
            state: CharacterState::None,
            mode: CharacterMode::General,
            has_moved: true,
            position: Default::default(),
            direction: Default::default(),
            velocity: 1.0,
            attachments: Attachments::new().await
        }
    }

    pub fn update(&mut self, scene: &mut Scene, terrain: &Terrain) {
        let mut ground_position = None;
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                if self.has_moved {
                                    match ground_position {
                                        Some(position) => instance.translate(&position),
                                        None => {
                                            let position = instance.set_on_the_ground(terrain);
                                            ground_position = Some(position);
                                        }
                                    }
                                }
                            }
                        },
                        _ => ()
                    };
                }
            }
        }
        if let Some(position) = ground_position {
            self.position = position;
        }
        self.attachments.weapon.update(&self, scene);
        if let Some(hair) = &self.attachments.hair {
            hair.update(&self, scene);
        }
        self.has_moved = false;
    }

    pub fn get_equip_right_matrix(&self, scene: &Scene) -> Option<Matrix4<f32>> {
        let (object_id, instance_id) = &self.objects[0];
        let object = scene.get(object_id).unwrap();
        let object3d = object.object3d.as_ref().unwrap();
        match object3d {
            Object3D::Skinned(skinned) => {
                let instance = skinned.get_immutable_instance(&instance_id).unwrap();
                if let Some(idx) = instance.skeleton.equip_right {
                    return Some(Matrix4::from(instance.skeleton.bones[idx].matrix_world))
                }
            },
            _ => (),
        }
        None
    }

    pub fn get_matrix(&self, scene: &Scene) -> Matrix4<f32> {
        let (object_id, instance_id) = &self.objects[0];
        let object = scene.get(object_id).unwrap();
        let object3d = object.object3d.as_ref().unwrap();
        match object3d {
            Object3D::Skinned(skinned) => {
                let instance = skinned.get_immutable_instance(&instance_id).unwrap();
                instance.get_matrix()
            },
            Object3D::Simple(simple) => {
                let instance = simple.get_immutable_instance(&instance_id).unwrap();
                instance.get_matrix()
            },
        }
    } 

    pub fn get_skeleton(&self, scene: &Scene) -> Option<Skeleton> {
        let (object_id, _) = &self.objects[0];
        let object = scene.get(object_id).unwrap();
        let object3d = object.object3d.as_ref().unwrap();
        match object3d {
            Object3D::Skinned(skinned) => {
                Some(clone_from_rc(skinned.skeleton.clone()))
            },
            Object3D::Simple(_) => {
                None
            },
        }
    }

    pub fn get_skeleton_instance(&self, scene: &Scene) -> Option<SkeletonInstance> {
        let (object_id, instance_id) = &self.objects[0];
        let object = scene.get(object_id).unwrap();
        let object3d = object.object3d.as_ref().unwrap();
        match object3d {
            Object3D::Skinned(skinned) => {
                let instance = skinned.get_immutable_instance(&instance_id).unwrap();
                Some(instance.skeleton.clone())
            },
            Object3D::Simple(_) => {
                None
            },
        }
    }

    pub fn get_mixer(&self, scene: &Scene) -> Option<AnimationMixer> {
        let (object_id, instance_id) = &self.objects[0];
        let object = scene.get(object_id).unwrap();
        let object3d = object.object3d.as_ref().unwrap();
        match object3d {
            Object3D::Skinned(skinned) => {
                let instance = skinned.get_immutable_instance(&instance_id).unwrap();
                Some(instance.mixer.clone())
            },
            Object3D::Simple(_) => {
                None
            },
        }
    }

    fn set_animation(&self, motion_name: &str, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                if let Some(motions_group) = self.motions.get_group(motion_name) {
                                    instance.mixer.queue(motions_group.clone());
                                }
                            }
                        },
                        _ => ()
                    };
                }
            }
        }
    }

    pub fn set_state(&mut self, state: CharacterState, scene: &mut Scene) {
        if self.state == state {
            return;
        }
        self.set_animation(&format!("{}_{}", self.mode, state), scene);
        self.state = state;
    }

    pub async fn set_attachment(&mut self, attachment_type: AttachmentType, id: &str, state: &mut State<'_>) {
        let objects = state.scene.create_instance_of(
            &match &attachment_type {
                AttachmentType::Weapon => format!("pack/item/weapon/{id}.glb"),
                AttachmentType::Hair => match &self.kind {
                    CharacterKind::PC(pc) => format!("pack/pc/{pc}/hair/{id}.glb"),
                    _ => return
                }
            },
            &state.device,
            &state.queue,
            &state.skinned_models_pipeline,
            &state.simple_models_pipeline,
            match &attachment_type {
                AttachmentType::Hair => Some(self.get_skeleton(&state.scene).unwrap()),
                _ => None,
            }
        ).await;
        let (object_id, instance_id) = &objects[0];
        match &attachment_type {
            AttachmentType::Weapon => {
                self.attachments.weapon = Weapon::Single(object_id.clone(), instance_id.clone());
            },
            AttachmentType::Hair => {
                if let Some(object) = state.scene.get_mut(object_id) {
                    object.matrix = Matrix4::from_scale(1.0).into(); // <- TODO: I should not do that
                }
                self.attachments.hair = Some(Hair(object_id.clone(), instance_id.clone()));
            }
        }
    }

    pub fn set_mode(&mut self, mode: CharacterMode) {
        self.mode = mode;
    }

    pub fn move_in_direction(&mut self, direction: Vector3<f32>, scene: &mut Scene, delta_ms: f32) {
        let normalized_direction = direction.normalize();
        let movement = normalized_direction * self.velocity * (delta_ms / 200.0);
        self.additive_translation(movement.x, movement.y, movement.z, scene);
        let rotation = Quaternion::from_arc(Vector3::new(0.0, 0.0, 1.0), direction, None);
        self.rotate(
            rotation.s, 
            rotation.v.x, 
            rotation.v.y, 
            rotation.v.z,
            scene
        );
        self.has_moved = true;
    }

}

#[derive(PartialEq)]
pub enum CharacterState {
    None,
    Wait,
    Run,
    Attack,
}

impl fmt::Display for CharacterState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            CharacterState::None => "",
            CharacterState::Wait => "WAIT",
            CharacterState::Run => "RUN",
            CharacterState::Attack => "ATTACK",
        };
        write!(f, "{}", output)
    }
}

impl TranslateWithScene for Character {
    fn translate(&mut self, x: f32, y: f32, z: f32, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Simple(simple) => {
                            if let Some(instance) = simple.get_instance(&instance_id) {
                                instance.translate(&[x, y, z]);
                            }
                        },
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                instance.translate(&[x, y, z]);
                            }
                        }
                    };
                }
            }
        }
    }
}

impl AdditiveTranslationWithScene for Character {
    fn additive_translation(&mut self, x: f32, y: f32, z: f32, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Simple(_simple) => todo!(),
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                instance.additive_translation(&[x, y, z]);
                            }
                        }
                    };
                }
            }
        }
    }
}

impl RotateWithScene for Character {
    fn rotate(&mut self, w: f32, xi: f32, yj: f32, zk: f32, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Simple(simple) => {
                            if let Some(_instance) = simple.get_instance(&instance_id) {
                                
                            }
                        },
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                instance.rotate(&[w, xi, yj, zk]);
                            }
                        }
                    };
                }
            }
        }
    }
}

pub enum Sex {
    Male,
    Female
}

impl fmt::Display for Sex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Sex::Male => "m",
            Sex::Female => "w",
        };
        write!(f, "{}", output)
    }
}

pub enum PCType {
    Shaman(Sex),
    Sura(Sex),
    Ninja(Sex),
    Warrior(Sex)
}

impl fmt::Display for PCType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            PCType::Shaman(sex) => format!("shaman_{sex}"),
            PCType::Sura(sex) => format!("sura_{sex}"),
            PCType::Ninja(sex) => format!("assassin_{sex}"),
            PCType::Warrior(sex) => format!("warrior_{sex}"),
        };
        write!(f, "{}", output)
    }
}

pub enum NPCType {
    Normal,
    Monster,
}

impl fmt::Display for NPCType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            NPCType::Normal => "normal",
            NPCType::Monster => "monster",
        };
        write!(f, "{}", output)
    }
}

pub enum CharacterKind {
    PC(PCType),
    NPC(NPCType),
}

impl fmt::Display for CharacterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            CharacterKind::PC(pc) => format!("pc/{}", &*pc.to_string()),
            CharacterKind::NPC(npc) => npc.to_string(),
        };
        write!(f, "{}", output)
    }
}

pub trait GetActor {
    fn get_actor(&mut self, actor: &Actor) -> &mut Character;
}

impl GetActor for Vec<Character> {
    fn get_actor(&mut self, actor: &Actor) -> &mut Character {
        self.iter_mut().find(|v| v.id == actor.character).unwrap()
    }
}

pub enum CharacterMode {
    General,
    Bell,
    Fan,
    Sword,
    TwohandSword,
}

impl fmt::Display for CharacterMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            CharacterMode::General => "GENERAL",
            CharacterMode::Bell => "BELL",
            CharacterMode::Fan => "FAN",
            CharacterMode::Sword => "ONEHANDSWORD",
            CharacterMode::TwohandSword => "TWOHANDSWORD",
        };
        write!(f, "{}", output)
    }
}