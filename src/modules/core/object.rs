use cgmath::SquareMatrix;
use crate::modules::utils::id_gen::generate_unique_string;
use super::object_3d::Object3D;

#[derive(Debug)]
pub struct Metadata {
    /// Used to declare some Object as Bone after parsing
    pub gltf_node_index: Option<usize>
}

#[derive(Debug)]
pub struct Object {
    pub id: String,
    pub name: Option<String>,
    pub parent: Option<String>,
    pub childrens: Vec<String>,
    pub matrix: [[f32; 4]; 4],
    pub matrix_world: [[f32; 4]; 4],
    #[allow(dead_code)]
    matrix_world_needs_update: bool,
    pub object3d: Option<Object3D>,
    pub metadata: Option<Metadata>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            id: generate_unique_string(),
            name: None,
            parent: None,
            childrens: Vec::new(),
            matrix: cgmath::Matrix4::identity().into(),
            matrix_world: cgmath::Matrix4::identity().into(),
            matrix_world_needs_update: false,
            object3d: None,
            metadata: None,
        }
    }
    pub fn set_object_3d(&mut self, object_3d: Object3D) {
        self.object3d = Some(object_3d);
    }
    pub fn get_object_3d(&mut self) -> Option<&mut Object3D> {
        self.object3d.as_mut()
    }
    pub fn add_child(&mut self, child: &mut Object) {
        child.parent = Some(self.id.clone());
        self.childrens.push(child.id.clone());
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