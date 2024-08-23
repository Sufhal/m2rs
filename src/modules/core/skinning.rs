use std::{collections::HashMap, rc::Rc};
use cgmath::{Decomposed, InnerSpace, Matrix4, Quaternion, SquareMatrix};

#[repr(C, align(16))]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct Mat4x4([[f32; 4]; 4]);

/// ### /!\ World matrices in Bones are in the Skeleton context, not in the Scene context
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
}
impl Bone {
    pub fn new(
        parent_index: Option<usize>, 
        name: Option<String>,
        inverse_bind_matrix: [[f32; 4]; 4],
        translation: &[f32; 3],
        rotation: &[f32; 4],
        scale: &[f32; 3],
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
}

impl Skeleton {
    pub fn create_instance(&self) -> SkeletonInstance {
        let mut instance = SkeletonInstance {
            bones: self.bones.clone(),
        };
        instance.calculate_world_matrices();
        instance
    }

    /// Returns each bones inversed bind matrix
    pub fn to_raw(&self) -> Vec<Mat4x4> {
        self.bones.iter().map(|bone| Mat4x4(bone.inverse_bind_matrix)).collect::<Vec<_>>()
    }

}

#[derive(Clone, Debug)]
/// Used by instances
pub struct SkeletonInstance {
    pub bones: Vec<Bone>,
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

const DEFAULT_CLIP_TRANSITION_DURATION_MS: f64 = 800.0;

#[derive(Clone, Debug)]
struct PlayState {
    animation: usize,
    elapsed_time: f64
}
#[derive(Clone, Debug)]
struct TransitionState {
    elapsed_time: f64,
    animation_in: usize,
    animation_out: usize
}
#[derive(Clone, Debug)]
enum MixerState {
    None,
    Play(PlayState),
    Transition(TransitionState)
}

#[derive(Clone, Debug)]
pub struct AnimationMixer {
    clips: Rc<Vec<AnimationClip>>,
    state: MixerState,
}

impl AnimationMixer {
    pub fn new(clips: Rc<Vec<AnimationClip>>, autoplay: bool) -> Self {
        Self {
            clips,
            state: match autoplay {
                true => MixerState::Play(PlayState { animation: 0, elapsed_time: 0.0 }),
                false => MixerState::None
            }
        }
    }
    fn find_animation(&self, clip_name: &str) -> Option<usize> {
        self.clips.iter().position(|c| c.name == clip_name)
    }
    pub fn update(&mut self, delta_ms: f64) {
        match &mut self.state {
            MixerState::Play(state) => {
                if state.elapsed_time + delta_ms > (self.clips[state.animation].duration * 1000.0) as f64 {
                    state.elapsed_time = 0.0; // loop
                } else {
                    state.elapsed_time += delta_ms;
                }
            },
            _ => ()
        };
    }
    pub fn play(&mut self, clip_name: &str) {
        if let Some(clip) = self.find_animation(clip_name) {
            match &mut self.state {
                MixerState::None => {
                    self.state = MixerState::Play(PlayState { animation: clip, elapsed_time: 0.0 });
                },
                MixerState::Play(state) => {
                    *state = PlayState { animation: clip, elapsed_time: 0.0 };
                },
                _ => ()
            };
        }
    }
    pub fn apply_on_skeleton(&self, skeleton: &mut SkeletonInstance) {
        match &self.state {
            MixerState::Play(state) => {
                let elapsed_secs = state.elapsed_time / 1000.0;
                let clip = &self.clips[state.animation];
                let timestamps = &clip.animations[0].timestamps;
                let timestamp_index = timestamps
                    .iter()
                    .enumerate()
                    .fold(0, |mut acc, (idx, t)| {
                        if (elapsed_secs - *t).abs() <= (elapsed_secs - timestamps[acc]).abs() {
                            acc = idx;
                        }
                        acc
                    });
                for bone_animation in &clip.animations {
                    let bone = &mut skeleton.bones[bone_animation.bone];
                    if bone_animation.timestamps.len() == 0 { return; }
                    match &bone_animation.keyframes {
                        Keyframes::Translation(frames) => {
                            bone.set_translation(&frames[timestamp_index]);
                        },
                        Keyframes::Rotation(frames) => {
                            bone.set_rotation(&frames[timestamp_index]);
                        },
                        Keyframes::Scale(frames) => {
                            bone.set_scale(&frames[timestamp_index]);
                        },
                        _ => {},
                    };
                }
                skeleton.calculate_world_matrices();
            },
            _ => ()
        };
        
    }
 
}