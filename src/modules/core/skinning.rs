use std::collections::HashMap;

use cgmath::{Matrix4, SquareMatrix};

/// ### /!\ World matrices in Bones are in the Skeleton context, not in the Scene context
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: Option<String>,
    pub parent_index: Option<usize>,
    pub translation: [f32; 3],
    pub bind_matrix: [[f32; 4]; 4],
    pub bind_matrix_world: [[f32; 4]; 4],
    pub inverse_bind_matrix_world: [[f32; 4]; 4],
    pub matrix_world: [[f32; 4]; 4],
}

impl Bone {
    pub fn new(parent_index: Option<usize>, bind_matrix: [[f32; 4]; 4], name: Option<String>) -> Self {
        Self {
            name,
            parent_index,
            translation: [0.0, 0.0, 0.0],
            bind_matrix,
            bind_matrix_world: Matrix4::identity().into(),
            inverse_bind_matrix_world: Matrix4::identity().into(),
            matrix_world: Matrix4::identity().into(),
        }
    }
    pub fn set_translation(&mut self, translation: &[f32; 3]) {
        for i in 0..3usize {
            self.translation[i] = translation[i];
        }
    }
}

#[derive(Clone, Debug)]
/// Created by model loaders as base
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

impl Skeleton {
    pub fn create_instance(&self) -> SkeletonInstance {
        SkeletonInstance {
            bones: self.bones.clone(),
        }
    }
    pub fn calculate_world_matrices(&mut self) {
        for i in 0..self.bones.len() {
            if self.bones[i].parent_index.is_none() {
                // Si le bone n'a pas de parent, il s'agit du bone racine.
                let identity = Matrix4::identity();
                self.calculate_world_matrix(i, &identity, &identity);
            }
        }
    }
    fn calculate_world_matrix(&mut self, bone_index: usize, parent_world_matrix: &Matrix4<f32>, parent_bind_world_matrix: &Matrix4<f32>) {
        let bone = &mut self.bones[bone_index];
        
        // bind
        let local_bind_matrix = Matrix4::from(bone.bind_matrix);
        let world_bind_matrix = parent_bind_world_matrix * local_bind_matrix;
        bone.bind_matrix_world = world_bind_matrix.into();
        bone.inverse_bind_matrix_world = world_bind_matrix.invert().unwrap_or(Matrix4::identity()).into();

        // current
        let local_matrix = local_bind_matrix * Matrix4::from_translation(bone.translation.into());
        let world_matrix = parent_world_matrix * local_matrix;
        bone.matrix_world = world_matrix.into();

        dbg!(&bone.matrix_world);

        // Appliquer récursivement aux enfants
        for j in 0..self.bones.len() {
            if let Some(parent_index) = self.bones[j].parent_index {
                if parent_index == bone_index {
                    self.calculate_world_matrix(j, &world_matrix, &world_bind_matrix);
                }
            }
        }
    }
    /// Returns each bones inversed bind matrix
    pub fn to_raw(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.inverse_bind_matrix_world).collect::<Vec<_>>()
    }
    pub fn to_raw_transform(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.matrix_world).collect::<Vec<_>>()
    }
}


/// Used by instances
pub struct SkeletonInstance {
    pub bones: Vec<Bone>,
}

impl SkeletonInstance {
    /// Returns each bones transform matrix based on current animation
    pub fn to_raw(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.matrix_world).collect::<Vec<_>>()
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
    pub keyframes: Keyframes,
    pub timestamps: Vec<f32>,
}

#[derive(Clone, Debug)]
pub enum Keyframes {
    Translation(Vec<Vec<f32>>),
    Other,
}