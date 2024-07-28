use std::collections::{HashMap, VecDeque};
use cgmath::SquareMatrix;

use super::object::{self, Object};
use super::object_3d::{self, Object3D};
use super::model::DrawModel;

pub struct Scene {
    objects: HashMap<String, Object>,
    root: String,
}

impl Scene {
    pub fn new() -> Self {
        let mut objects = HashMap::new();
        let root_object = Object::new();
        let root = root_object.id.clone();
        objects.insert(root.clone(), root_object);
        Self {
            objects,
            root,
        }
    }
    /// Add `Object` to internal Scene HashMap, it will set `root` as parent if None was defined
    pub fn add(&mut self, mut object: Object) {
        if let None = object.parent {
            object.parent = Some(self.root.clone());
        }
        self.objects.insert(object.id.clone(), object);
    }
    pub fn remove(&mut self, object_id: &str) -> Option<Object> {
        self.objects.remove(object_id)
    }
    pub fn get(&self, object_id: &str) -> Option<&Object> {
        self.objects.get(object_id)
    }
    pub fn get_mut(&mut self, object_id: &str) -> Option<&mut Object> {
        self.objects.get_mut(object_id)
    }
    pub fn get_root(&mut self) -> &mut Object {
        self.objects.get_mut(&self.root).unwrap()
    }
    pub fn get_all_objects(&mut self) -> Vec<&mut Object> {
        self.objects
            .iter_mut()
            .map(|(_, object)| object)
            .collect::<Vec<_>>()
    }
    pub fn compute_world_matrices(&mut self) {
        let mut queue = VecDeque::new();
        queue.push_back(self.root.clone());
        while let Some(current_id) = queue.pop_front() {
            if let Some(current_object) = self.objects.get(&current_id) {
                let parent_world_transform = if let Some(parent_id) = &current_object.parent {
                    self.objects.get(parent_id).map_or(cgmath::Matrix4::identity(), |parent| parent.matrix_world.into())
                } else {
                    cgmath::Matrix4::identity()
                };
                if let Some(current_object) = self.objects.get_mut(&current_id) {
                    current_object.matrix_world = (parent_world_transform * cgmath::Matrix4::from(current_object.matrix)).into();
                    // Add children to the queue
                    for (child_id, child_object) in self.objects.iter() {
                        if child_object.parent.as_ref() == Some(&current_id) {
                            queue.push_back(child_id.clone());
                        }
                    }
                }
            }
        }
    }
}

pub trait DrawScene<'a> {
    fn draw_scene(
        &mut self,
        queue: &wgpu::Queue,
        scene: &'a mut Scene,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawScene<'b> for wgpu::RenderPass<'a>
where 
    'b: 'a,
{
    fn draw_scene(
        &mut self,
        queue: &wgpu::Queue,
        scene: &'b mut Scene,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for object in scene.get_all_objects() {
            if let Some(object_3d) = object.get_object_3d() {
                object_3d.update_instances(queue);
                self.set_vertex_buffer(1, object_3d.get_instance_buffer_slice());
                self.draw_model_instanced(
                    &object_3d.model, 
                    0..object_3d.get_taken_instances_count() as u32, 
                    camera_bind_group, 
                    light_bind_group
                );
            }
        }
    }
}