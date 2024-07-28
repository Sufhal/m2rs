use cgmath::SquareMatrix;
use crate::modules::utils::id_gen::generate_unique_string;

use super::{object_3d::{self, Object3D}, scene::Scene};

pub struct Object {
    pub id: String,
    pub parent: Option<String>,
    pub childrens: Vec<String>,
    pub matrix: [[f32; 4]; 4],
    pub matrix_world: [[f32; 4]; 4],
    matrix_world_needs_update: bool,
    object_3d: Option<Object3D>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            id: generate_unique_string(),
            parent: None,
            childrens: Vec::new(),
            matrix: cgmath::Matrix4::identity().into(),
            matrix_world: cgmath::Matrix4::identity().into(),
            matrix_world_needs_update: false,
            object_3d: None,
        }
    }
    pub fn set_object_3d(&mut self, object_3d: Object3D) {
        self.object_3d = Some(object_3d);
    }
    pub fn get_object_3d(&mut self) -> Option<&mut Object3D> {
        self.object_3d.as_mut()
    }
    pub fn add_child(&mut self, mut child: Object) {
        child.parent = Some(self.id.clone());
        self.childrens.push(child.id);
    }
    pub fn compute_matrix_world(&mut self, parent_matrix: Option<[[f32; 4]; 4]>) {
        if let Some(parent_matrix) = parent_matrix {
            let parent_matrix_world = cgmath::Matrix4::from(parent_matrix);
            let local_matrix = cgmath::Matrix4::from(self.matrix);
            self.matrix_world = (parent_matrix_world * local_matrix).into();
        } else {
            self.matrix_world = self.matrix;
        }
    }
}