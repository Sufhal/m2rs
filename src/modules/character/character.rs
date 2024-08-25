use std::fmt;
use crate::modules::assets::gltf_loader::load_model_glb;
use crate::modules::core::motions::MotionsGroup;
use crate::modules::core::object::Object;
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
    kind: CharacterKind,
    instance_ids: Vec<String>,
    motions: MotionsGroup
}

impl Character {
    pub async fn load<'a>(name: &str, kind: CharacterKind, state: &mut State<'a>) -> Self {
        let instance_ids = if let Some(assets) = state.scene.get_childrens_of(name) {
            // object is already loaded, we just have to create instances
            let mut ids = Vec::new();
            for asset in assets {
                let object3d = asset.object_3d.as_mut().expect("Loading a Character requires that Object's name inherits from its Object3D's name");
                let instance = object3d.request_instance(&state.device);
                instance.take();
                ids.push(instance.id.clone());
            }
            ids
        } else {
            // object needs to be created to create an instance
            let mut ids = Vec::new();
            let filename = match &kind {
                CharacterKind::NPC(npc) => {
                    let npc_type = npc.to_string();
                    format!("pack/{npc_type}/{name}/{name}.glb")
                },
                CharacterKind::PC(pc) => {
                    let pc_type = pc.to_string();
                    format!("pack/{pc_type}/{name}.glb")
                }
            };
            let model_objects = load_model_glb(
                &filename,
                &state.device,
                &state.queue,
                &state.new_render_pipeline
            ).await.expect("unable to load");
            let mut group = Object::new();
            for mut object in model_objects {
                group.add_child(&mut object);
                if let Some(object_3d) = &mut object.object_3d {
                    let instance = object_3d.request_instance(&state.device);
                    instance.take();
                    ids.push(instance.id.clone());
                }
                state.scene.add(object);
            }
            state.scene.add(group);
            ids
        };
        let motions = match &kind {
            CharacterKind::NPC(npc) => {
                match npc {
                    NPCType::Monster => {
                        let npc_type = npc.to_string();
                        let filename = format!("pack/{npc_type}/{name}/motion.json");
                        MotionsGroup::load(&filename).await.unwrap()
                    },
                    NPCType::Normal => todo!()
                }
            },
            CharacterKind::PC(_) => todo!()
        };
        Self {
            kind,
            instance_ids,
            motions
        }
    }
}