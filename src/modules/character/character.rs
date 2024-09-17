use std::fmt::{self};
use crate::modules::assets::gltf_loader::{load_animation, load_model_glb};
use crate::modules::core::motions::MotionsGroups;
use crate::modules::core::object::Object;
use crate::modules::core::object_3d::{Object3D, Rotate, RotateWithScene, Translate, TranslateWithScene};
use crate::modules::core::scene::Scene;
use crate::modules::state::State;

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

pub struct Character {
    #[allow(dead_code)]
    kind: CharacterKind,
    pub objects: Vec<(String, String)>, // (Object ID, Object3DInstance ID)
    motions: MotionsGroups
}

impl Character {
    pub async fn new<'a>(name: &str, kind: CharacterKind, state: &mut State<'a>) -> Self {
        Self::load(name, kind, state).await
    }
    async fn load<'a>(name: &str, kind: CharacterKind, state: &mut State<'a>) -> Self {
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
                    _ => todo!()
                }
            }
        };
        let objects = if let Some(childrens) = state.scene.get_childrens_of(name) {
            // object is already loaded, we just have to create instances
            let mut objects = Vec::new();
            for children in childrens {
                if let Some(object) = state.scene.get_mut(&children) {
                    if let Some(object3d) = &mut object.object3d {
                        match object3d {
                            Object3D::Simple(simple) => {
                                let instance = simple.request_instance(&state.device);
                                instance.take();
                                objects.push((object.id.clone(), instance.id.clone()));
                            },
                            Object3D::Skinned(skinned) => {
                                let instance = skinned.request_instance(&state.device);
                                instance.take();
                                objects.push((object.id.clone(), instance.id.clone()));
                            },
                        };
                    }
                }
            }
            objects
        } else {
            // object needs to be created to create an instance
            let mut objects = Vec::new();
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
            let model_objects = load_model_glb(
                &filename,
                &state.device,
                &state.queue,
                &state.skinned_models_pipeline,
                &state.simple_models_pipeline,
            ).await.expect("unable to load");
            let mut group = Object::new();
            group.name = Some(name.to_string());
            for mut object in model_objects {
                group.add_child(&mut object);
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Simple(simple) => {
                            // create instance
                            let instance = simple.request_instance(&state.device);
                            instance.take();
                            objects.push((object.id.clone(), instance.id.clone()));
                        },
                        Object3D::Skinned(skinned) => {
                            dbg!(name);
                            // loading animations clips attached to motions
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
                                    let clip = load_animation(&path, name).await.unwrap();
                                    dbg!(&clip.name);
                                    skinned.add_animation(clip);
                                }
                            }
                            // create instance
                            let instance = skinned.request_instance(&state.device);
                            instance.take();
                            objects.push((object.id.clone(), instance.id.clone()));
                        },
                    }
                }
                state.scene.add(object);
            }
            state.scene.add(group);
            objects
        };

        Self {
            kind,
            objects,
            motions
        }
    }

    pub fn update(&self, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                

                            }
                        },
                        _ => ()
                    };
                }
            }
        }
    }

    pub fn set_animation(&self, motion_name: &str, scene: &mut Scene) {
        for (object_id, instance_id) in &self.objects {
            if let Some(object) = scene.get_mut(object_id) {
                if let Some(object3d) = &mut object.object3d {
                    match object3d {
                        Object3D::Skinned(skinned) => {
                            if let Some(instance) = skinned.get_instance(&instance_id) {
                                if let Some(motions_group) = self.motions.get_group(motion_name) {
                                    instance.mixer.play(motions_group.clone());
                                }
                            }
                        },
                        _ => ()
                    };
                }
            }
        }
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