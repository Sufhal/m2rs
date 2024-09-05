use cgmath::SquareMatrix;
use crate::modules::utils::id_gen::generate_unique_string;
use super::object_3d::Object3D;

#[derive(Debug)]
pub struct Object {
    pub id: String,
    pub name: Option<String>,
    pub parent: Option<String>,
    pub childrens: Vec<String>,
    pub matrix: [[f32; 4]; 4],
    pub matrix_world: [[f32; 4]; 4],
    pub object3d: Option<Object3D>,
    pub matrix_world_needs_update: bool,
    pub matrix_world_buffer_needs_update: bool,
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
            matrix_world_needs_update: true,
            matrix_world_buffer_needs_update: false,
            object3d: None,
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

}