use std::collections::{HashMap, VecDeque};
use cgmath::{Matrix4, SquareMatrix};
use crate::modules::pipelines::common_pipeline::CommonPipeline;
use crate::modules::pipelines::simple_models_pipeline::SimpleModelPipeline;
use crate::modules::pipelines::skinned_models_pipeline::{self, SkinnedModelPipeline};
use super::object::Object;
use super::model::{DrawSimpleModel, DrawSkinnedModel, TransformUniform};
use super::object_3d::Object3D;

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
            self.get_root().childrens.push(object.id.clone());
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
    pub fn get_all_objects(&self) -> Vec<&Object> {
        self.objects
            .iter()
            .map(|(_, object)| object)
            .collect::<Vec<_>>()
    }
    pub fn get_all_objects_mut(&mut self) -> Vec<&mut Object> {
        self.objects
            .iter_mut()
            .map(|(_, object)| object)
            .collect::<Vec<_>>()
    }
    pub fn get_childrens_of(&mut self, object_name: &str) -> Option<Vec<String>> {
        let object = self.objects
            .iter()
            .filter(|(_, object)| object.name.is_some())
            .find(|(_, object)| object.name.as_ref().unwrap().as_str() == object_name);
        if let Some((_, object)) = &object {
            Some(object.childrens.clone())
        } else {
            None
        }
    }
    pub fn compute_world_matrices(&mut self) {
        let mut queue = VecDeque::with_capacity(self.objects.len());
        queue.push_back((self.root.clone(), false));
        while let Some((current_id, force)) = queue.pop_front() {
            if let Some(current_object) = self.objects.get(&current_id) {
                let parent_world_transform = if let Some(parent_id) = &current_object.parent {
                    self.objects.get(parent_id).map_or(Matrix4::identity(), |parent| parent.matrix_world.into())
                } else {
                    Matrix4::identity()
                };
                if let Some(current_object) = self.objects.get_mut(&current_id) {
                    let needs_update = current_object.matrix_world_needs_update || force;
                    if needs_update {
                        current_object.matrix_world = (parent_world_transform * Matrix4::from(current_object.matrix)).into();
                        current_object.matrix_world_needs_update = false;
                        current_object.matrix_world_buffer_needs_update = true;
                    }
                    for children in &current_object.childrens {
                        queue.push_back((children.clone(), needs_update));
                    }
                }
            }
        }
    }
    pub fn update_objects_buffers(&mut self, queue: &wgpu::Queue) {
        for (_, object) in &mut self.objects {
            if !object.matrix_world_buffer_needs_update { continue }
            object.matrix_world_buffer_needs_update = false;
            if let Some(object3d) = &object.object3d {
                match object3d {
                    Object3D::Simple(simple) => {
                        for mesh in &simple.model.meshes {
                            queue.write_buffer(&mesh.transform_buffer, 0, bytemuck::cast_slice(&[TransformUniform::from(object.matrix_world)]));
                        }
                    },
                    Object3D::Skinned(skinned) => {
                        for mesh in &skinned.model.meshes {
                            queue.write_buffer(&mesh.transform_buffer, 0, bytemuck::cast_slice(&[TransformUniform::from(object.matrix_world)]));
                        }
                    }
                }
                
            }
        }
    }
}

pub trait DrawScene<'a> {
    fn draw_scene_simple_models(
        &mut self,
        scene: &'a Scene,
        common_pipeline: &'a CommonPipeline,
    );
    fn draw_scene_skinned_models(
        &mut self,
        scene: &'a Scene,
        common_pipeline: &'a CommonPipeline,
    );
}

impl<'a, 'b> DrawScene<'b> for wgpu::RenderPass<'a>
where 
    'b: 'a,
{

    fn draw_scene_simple_models(
        &mut self,
        scene: &'b Scene,
        common_pipeline: &'a CommonPipeline,
    ) {
        for object in scene.get_all_objects() {
            if let Some(object3d) = &object.object3d {
                match object3d {
                    Object3D::Simple(simple) => {
                        // simple.update_instances(queue);
                        self.set_vertex_buffer(1, simple.get_instance_buffer_slice());
                        self.draw_simple_model_instanced(
                            &simple.model, 
                            &simple.instances_bind_group,
                            0..simple.get_taken_instances_count() as u32, 
                            common_pipeline
                        );
                    },
                    _ => ()
                }
            }
        }
    }

    fn draw_scene_skinned_models(
        &mut self,
        scene: &'b Scene,
        common_pipeline: &'a CommonPipeline,
    ) {
        for object in scene.get_all_objects() {
            if let Some(object3d) = &object.object3d {
                match object3d {
                    Object3D::Skinned(skinning) => {
                        // skinning.update_instances(queue);
                        self.set_vertex_buffer(1, skinning.get_instance_buffer_slice());
                        self.draw_skinned_model_instanced(
                            &skinning.model, 
                            &skinning.instances_bind_group,
                            0..skinning.get_taken_instances_count() as u32, 
                            common_pipeline
                        );
                    },
                    _ => ()
                }
            }
        }
    }

}