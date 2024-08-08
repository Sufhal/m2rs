use std::collections::HashMap;

use cgmath::{Matrix4, SquareMatrix};

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: Option<String>,
    pub parent_index: Option<usize>,
    pub matrix: [[f32; 4]; 4],
    pub bind_matrix: [[f32; 4]; 4],
    pub inverse_bind_matrix: [[f32; 4]; 4],
    pub transform_matrix: [[f32; 4]; 4],
}

impl Bone {
    pub fn new(parent_index: Option<usize>, matrix: [[f32; 4]; 4], name: Option<String>) -> Self {
        Self {
            name,
            parent_index,
            matrix,
            transform_matrix: Matrix4::identity().into(),
            bind_matrix: Matrix4::identity().into(),
            inverse_bind_matrix: Matrix4::identity().into(),
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
    pub fn compute_bind_matrices(&mut self) {
        for i in 0..self.bones.len() {
            self.compute_bind_matrix(i);
        }
    }
    fn compute_bind_matrix(&mut self, index: usize) {
        let parent_matrix = if let Some(parent_index) = self.bones[index].parent_index {
            self.bones[parent_index].bind_matrix.into()
        } else {
            Matrix4::identity()
        };
        let local_transform = Matrix4::from(self.bones[index].matrix);
        self.bones[index].bind_matrix = (parent_matrix * local_transform).into();
        self.bones[index].inverse_bind_matrix = Matrix4::from(self.bones[index].bind_matrix).invert().unwrap().into();
    }
    /// Returns each bones inversed bind matrix
    pub fn to_raw(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.inverse_bind_matrix).collect::<Vec<_>>()
    }
    pub fn to_raw_transform(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.bind_matrix).collect::<Vec<_>>()
    }
}


/// Used by instances
pub struct SkeletonInstance {
    pub bones: Vec<Bone>,
}

impl SkeletonInstance {
    /// Returns each bones transform matrix based on current animation
    pub fn to_raw(&self) -> Vec<[[f32; 4]; 4]> {
        self.bones.iter().map(|bone| bone.transform_matrix).collect::<Vec<_>>()
    }
}

pub struct Animation {
    pub keyframes: HashMap<String, Vec<Keyframe>>,
}

pub struct Keyframe {
    pub timestamp: f32,
    pub bones_transforms: HashMap<String, [[f32; 4]; 4]>,
}