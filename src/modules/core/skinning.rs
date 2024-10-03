use std::{cell::RefCell, collections::HashMap, rc::Rc};
use cgmath::{InnerSpace, Matrix4, Quaternion, SquareMatrix};
use crate::modules::utils::functions::{clamp_f64, denormalize_f32x3, denormalize_f32x4, interpolate_rotations, normalize_f64};
use super::motions::MotionsGroup;

const ANIMATION_TRANSITION_DURATION: f64 = 150.0;

#[repr(C, align(16))]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct Mat4x4([[f32; 4]; 4]);

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct SkinningInformations {
    bones_count: u32,
}

/// ### ⚠️ World matrices in Bones are in the Skeleton context, not in the Scene context
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: Option<String>,
    pub parent_index: Option<usize>,
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub bind_matrix: [[f32; 4]; 4],
    pub inverse_bind_matrix: [[f32; 4]; 4],
    pub matrix_world: [[f32; 4]; 4],
    pub childrens: Vec<usize>,
}
impl Bone {
    pub fn new(
        parent_index: Option<usize>, 
        name: Option<String>,
        inverse_bind_matrix: [[f32; 4]; 4],
        translation: &[f32; 3],
        rotation: &[f32; 4],
        scale: &[f32; 3],
        childrens: Vec<usize>,
    ) -> Self {
        Self {
            name,
            parent_index,
            translation: *translation,
            rotation: *rotation,
            scale: *scale,
            bind_matrix: (
                Matrix4::from_translation((*translation).into()) *
                Matrix4::from(Quaternion::from(*rotation).normalize()) *
                Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2])
            ).into(),
            inverse_bind_matrix, 
            matrix_world: Matrix4::identity().into(),
            childrens,
        }
    }
    pub fn set_translation(&mut self, translation: &[f32; 3]) {
        self.translation = translation.clone();
    }
    pub fn set_rotation(&mut self, rotation: &[f32; 4]) {
        self.rotation = rotation.clone();
    }
    pub fn set_scale(&mut self, scale: &[f32; 3]) {
        self.scale = scale.clone();
    }
}

#[derive(Clone, Debug)]
/// Created by model loaders as base
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub equip_right: Option<usize>,
    pub equip_left: Option<usize>,
}

impl Skeleton {
    pub fn new(bones: Vec<Bone>) -> Self {
        let skeleton = Self {
            equip_left: None,
            equip_right: None,
            bones,
        };
        skeleton
    }
    pub fn create_instance(&self) -> SkeletonInstance {
        let mut instance = SkeletonInstance {
            bones: self.bones.clone(),
            equip_right: self.equip_right.clone(),
            equip_left: self.equip_left.clone(),
        };
        instance.calculate_world_matrices();
        instance
    }

    pub fn reorder_based_on_existing(&mut self, existing: &Skeleton) {
        // Créer un hashmap pour récupérer l'index des bones de l'existant par leur nom
        let existing_bone_index_map: HashMap<&String, usize> = existing
            .bones
            .iter()
            .enumerate()
            .filter_map(|(i, bone)| bone.name.as_ref().map(|name| (name, i)))
            .collect();

        // Trier les bones du nouveau skeleton en fonction de l'ordre des indices de l'existant
        self.bones.sort_by(|a, b| {
            let index_a = a.name.as_ref().and_then(|name| existing_bone_index_map.get(name));
            let index_b = b.name.as_ref().and_then(|name| existing_bone_index_map.get(name));
            index_a.cmp(&index_b)
        });

        // for i in 0..self.bones.len() {
            // let existing = existing.bones.get(i).map(|v| v.bind_matrix.clone());
            // let current = self.bones.get(i).map(|v| v.bind_matrix.clone());
            // println!("existing bone {i} is {:?} and self {:?}", existing, current);
        // }

    }

    /// Returns each bones inversed bind matrix
    pub fn to_raw_inverse_bind_matrices(&self) -> Vec<Mat4x4> {
        self.bones.iter().map(|bone| Mat4x4(bone.inverse_bind_matrix)).collect::<Vec<_>>()
    }

    pub fn to_raw_skinning_informations(&self) -> SkinningInformations {
        SkinningInformations {
            bones_count: self.bones.len() as u32
        }
    }

}

#[derive(Clone, Debug)]
/// Used by instances
pub struct SkeletonInstance {
    pub bones: Vec<Bone>,
    pub equip_right: Option<usize>,
    pub equip_left: Option<usize>,
}

impl SkeletonInstance {
    pub fn calculate_world_matrices(&mut self) {
        for i in 0..self.bones.len() {
            if self.bones[i].parent_index.is_none() {
                let identity = Matrix4::identity();
                self.calculate_world_matrix(i, &identity);
            }
        }
    }
    // TODO: optimization required
    fn calculate_world_matrix(&mut self, bone_index: usize, parent_world_matrix: &Matrix4<f32>) {
        let bone = &mut self.bones[bone_index];
        let transformation_matrix = 
            Matrix4::from_translation(bone.translation.into()) *
            Matrix4::from(Quaternion::from(bone.rotation).normalize()) *
            Matrix4::from_nonuniform_scale(bone.scale[0], bone.scale[1], bone.scale[2]);
        let world_matrix = parent_world_matrix * transformation_matrix;
        bone.matrix_world = world_matrix.into();
        for j in 0..self.bones.len() {
            if let Some(parent_index) = self.bones[j].parent_index {
                if parent_index == bone_index {
                    self.calculate_world_matrix(j, &world_matrix);
                }
            }
        }
    }
    pub fn to_raw_transform(&self) -> Vec<Mat4x4> {
        self.bones.iter().map(|bone| Mat4x4(bone.matrix_world)).collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug)]
pub struct AnimationClip {
    pub name: String,
    pub duration: f64,
    pub animations: Vec<BoneAnimation>
}

#[derive(Clone, Debug)]
pub struct BoneAnimation {
    pub bone: usize,
    // pub name: String,
    pub keyframes: Keyframes,
    pub timestamps: Vec<f64>,
}

#[derive(Clone, Debug)]
pub enum Keyframes {
    Translation(Vec<[f32; 3]>),
    Rotation(Vec<[f32; 4]>),
    Scale(Vec<[f32; 3]>),
    Other,
}

#[derive(Clone, Debug)]
pub struct PlayState {
    animation: usize,
    elapsed_time: f64,
}
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct BlendState {
    animation: usize,
    elapsed_time: f64,
}
#[derive(Clone, Debug)]
pub enum MixerState {
    None,
    Play(PlayState),
    Blend(BlendState)
}

#[derive(Clone, Debug)]
pub struct AnimationMixer {
    clips: Rc<RefCell<Vec<AnimationClip>>>,
    state: MixerState,
    current_motion_group: Option<MotionsGroup>,
    queued_motion_groups: Vec<MotionsGroup>,
}

impl AnimationMixer {
    pub fn new(clips: Rc<RefCell<Vec<AnimationClip>>>, autoplay: bool) -> Self {
        Self {
            clips,
            state: match autoplay {
                true => MixerState::Play(PlayState { animation: 0, elapsed_time: 0.0}),
                false => MixerState::None
            },
            current_motion_group: None,
            queued_motion_groups: Vec::new(),
        }
    }
    fn find_animation(&self, clip_name: &str) -> Option<usize> {
        let clips = RefCell::borrow(&self.clips);
        clips.iter().position(|c| c.name == clip_name)
    }
    pub fn update(&mut self, delta_ms: f64) {
        match &mut self.state {
            MixerState::Blend(state) => {
                state.elapsed_time += delta_ms;
                if state.elapsed_time > ANIMATION_TRANSITION_DURATION {
                    self.state = MixerState::Play(
                        PlayState { 
                            animation: state.animation, 
                            elapsed_time: state.elapsed_time 
                        }
                    );
                }
            },
            MixerState::Play(state) => {
                state.elapsed_time += delta_ms;
            },
            MixerState::None => {
                if self.queued_motion_groups.len() > 0 {
                    let queued = self.queued_motion_groups.remove(0);
                    self.play(queued);
                }
                else {
                    if let Some(motions_group) = &self.current_motion_group {
                        self.play(motions_group.clone());
                    }
                }
            }
        };
    }
    /// Add a motions group to the queue. 
    /// The queue is actually used depending the current motions group
    pub fn queue(&mut self, motions_groups: Vec<MotionsGroup>) {
        match &mut self.state {
            MixerState::Play(_) |
            MixerState::Blend(_) => {
                if let Some(current) = &self.current_motion_group {
                    if !animation_is_cancellable(&current.name) {
                        self.queued_motion_groups.extend(motions_groups);
                        return
                    }
                }
            },
            _ => (),
        };
        self.play(motions_groups[0].clone());
    }
    /// Play a motions group immediately
    pub fn play(&mut self, motions_group: MotionsGroup) {
        let motion = motions_group.pick_motion();
        if let Some(clip) = self.find_animation(&motion.file) {
            match &mut self.state {
                MixerState::None => {
                    self.state = MixerState::Play(
                        PlayState { 
                            animation: clip, 
                            elapsed_time: 0.0 
                        }
                    );
                },
                MixerState::Play(_) => {
                    self.state = MixerState::Blend(
                        BlendState {
                            animation: clip,
                            elapsed_time: 0.0,
                        }
                    );
                },
                MixerState::Blend(state) => {
                    state.animation = clip;
                    state.elapsed_time = 0.0;
                }
            };
        }
        if self.current_motion_group.is_none() || self.current_motion_group.as_ref().unwrap().name != motions_group.name {
            self.current_motion_group = Some(motions_group);
        }
    }
    pub fn apply_on_skeleton(&mut self, skeleton: &mut SkeletonInstance) {
        let clips = RefCell::borrow(&self.clips);
        match &self.state {
            MixerState::Play(state) => {
                let elapsed_secs = state.elapsed_time / 1000.0;
                let clip = &clips[state.animation];
                let timestamps = &clip.animations[0].timestamps;
                if let Some(next) = timestamps.iter().position(|t| *t > elapsed_secs) {
                    let previous = next - 1;
                    let factor = normalize_f64(elapsed_secs, timestamps[previous], timestamps[next]);
                    let factor = clamp_f64(factor, 0.0, 1.0);
                    for bone_animation in &clip.animations {
                        let bone = &mut skeleton.bones[bone_animation.bone];
                        match &bone_animation.keyframes {
                            Keyframes::Translation(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = denormalize_f32x3(factor as f32, previous_frame, next_frame);
                                bone.set_translation(&interpolated);
                            },
                            Keyframes::Rotation(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = interpolate_rotations(factor as f32, previous_frame, next_frame);
                                bone.set_rotation(&interpolated);
                            },
                            Keyframes::Scale(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = denormalize_f32x3(factor as f32, previous_frame, next_frame);
                                bone.set_scale(&interpolated);
                            },
                            _ => {},
                        };
                    }
                    skeleton.calculate_world_matrices();
                }
                else {
                    self.state = MixerState::None;
                }
            },
            MixerState::Blend(state) => {
                let transition_factor = normalize_f64(state.elapsed_time, 0.0, ANIMATION_TRANSITION_DURATION);
                let elapsed_secs = state.elapsed_time / 1000.0;
                let clip = &clips[state.animation];
                let timestamps = &clip.animations[0].timestamps;
                if let Some(next) = timestamps.iter().position(|t| *t > elapsed_secs) {
                    let previous = next - 1;
                    let factor = normalize_f64(elapsed_secs, timestamps[previous], timestamps[next]);
                    let factor = clamp_f64(factor, 0.0, 1.0);
                    for bone_animation in &clip.animations {
                        let bone = &mut skeleton.bones[bone_animation.bone];
                        match &bone_animation.keyframes {
                            Keyframes::Translation(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = denormalize_f32x3(factor as f32, previous_frame, next_frame);
                                let blent = denormalize_f32x3(transition_factor as f32, &bone.translation, &interpolated);
                                bone.set_translation(&blent);
                            },
                            Keyframes::Rotation(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = interpolate_rotations(factor as f32, previous_frame, next_frame);
                                let blent = interpolate_rotations(transition_factor as f32, &bone.rotation, &interpolated);
                                bone.set_rotation(&blent);
                            },
                            Keyframes::Scale(frames) => {
                                let previous_frame = &frames[previous];
                                let next_frame = &frames[next];
                                let interpolated = denormalize_f32x3(factor as f32, previous_frame, next_frame);
                                let blent = denormalize_f32x3(transition_factor as f32, &bone.scale, &interpolated);
                                bone.set_scale(&blent);
                            },
                            _ => {},
                        };
                    }
                    skeleton.calculate_world_matrices();
                }
                else {
                    self.state = MixerState::None;
                }
            }
            _ => ()
        };
    }


 
}

fn animation_is_cancellable(name: &str) -> bool {
    if name.contains("WAIT") { return true }
    if name.contains("RUN") { return true }
    false
}