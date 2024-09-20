use std::{cell::RefCell, rc::Rc};
use cgmath::{One, Vector3};
use wgpu::util::DeviceExt;
use crate::modules::{pipelines::{simple_models_pipeline::SimpleModelBindGroupLayouts, skinned_models_pipeline::SkinnedModelBindGroupLayouts}, terrain::terrain::Terrain, utils::id_gen::generate_unique_string};
use super::{instance::InstanceRaw, model::{SimpleModel, SkinnedModel}, raycaster::Raycaster, scene::Scene, skinning::{AnimationClip, AnimationMixer, Mat4x4, Skeleton, SkeletonInstance}};

type Vec3 = cgmath::Vector3<f32>;
type Quat = cgmath::Quaternion<f32>;

const INITIAL_INSTANCES_COUNT: usize = 100;

#[derive(Debug)]
pub struct SimpleObject3D {
    pub id: String,
    pub model: SimpleModel,
    pub instances_bind_group: wgpu::BindGroup,
    pub instances: Vec<SimpleObject3DInstance>,
    instances_buffer: wgpu::Buffer,
}

impl SimpleObject3D {
    pub fn request_instance(&mut self, device: &wgpu::Device) -> &mut SimpleObject3DInstance {
        if let Some(index) = self.find_available_instance() {
            return self.instances.get_mut(index).unwrap();
        } 
        self.increase_instances_capability(device);
        self.request_instance(device)
    }
    pub fn update_instances(&mut self, queue: &wgpu::Queue) {
        let size = std::mem::size_of::<InstanceRaw>();
        for (index, instance) in self.instances.iter_mut().enumerate() {
            if instance.needs_update == true {
                queue.write_buffer(
                    &self.instances_buffer,
                    (index * size) as wgpu::BufferAddress,
                    bytemuck::cast_slice(&[instance.to_instance_raw()]),
                );
                instance.needs_update = false;
            }
        }
    }
    pub fn get_instance(&mut self, id: &str) -> Option<&mut SimpleObject3DInstance> {
        self.instances.iter_mut().find(|i| &i.id == id)
    }
    pub fn get_instances(&mut self) -> &mut Vec<SimpleObject3DInstance> {
        &mut self.instances
    }
    pub fn find_available_instance(&self) -> Option<usize> {
        self.instances.iter().position(|v| v.busy == false)
    }
    pub fn increase_instances_capability(&mut self, device: &wgpu::Device) {
        let current_capacity = self.instances.capacity();
        let new_capacity = current_capacity * 2;
        self.instances.reserve(new_capacity - self.instances.capacity());
        for _ in 0..current_capacity {
            self.instances.push(SimpleObject3DInstance::new());
        }
        self.instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (new_capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
    pub fn get_taken_instances_count(&self) -> usize {
        self.instances.iter().filter(|i| i.busy == true).count()
    }
    pub fn get_instance_buffer_slice(&self) -> wgpu::BufferSlice {
        self.instances_buffer.slice(..)
    }
}

#[derive(Debug)]
pub struct SkinnedObject3D {
    pub id: String,
    pub model: SkinnedModel,
    pub instances_bind_group: wgpu::BindGroup,
    pub instances: Vec<SkinnedObject3DInstance>,
    instances_buffer: wgpu::Buffer,
    animation_clips: Rc<RefCell<Vec<AnimationClip>>>,
    skeleton: Rc<Skeleton>,
    skeletons_buffer: wgpu::Buffer,
    skeletons_bind_inverse_buffer: wgpu::Buffer,
}

impl SkinnedObject3D {
    pub fn request_instance(&mut self, device: &wgpu::Device) -> &mut SkinnedObject3DInstance {
        if let Some(index) = self.find_available_instance() {
            return self.instances.get_mut(index).unwrap();
        } 
        self.increase_instances_capability(device);
        self.request_instance(device)
    }
    pub fn update_instances(&mut self, queue: &wgpu::Queue) {
        let size = std::mem::size_of::<InstanceRaw>();
        for (index, instance) in self.instances.iter().enumerate() {
            if instance.needs_update == true {
                queue.write_buffer(
                    &self.instances_buffer,
                    (index * size) as wgpu::BufferAddress,
                    bytemuck::cast_slice(&[instance.to_instance_raw()]),
                );
            }
        }
    }
    pub fn get_instance(&mut self, id: &str) -> Option<&mut SkinnedObject3DInstance> {
        self.instances.iter_mut().find(|i| &i.id == id)
    }
    pub fn get_instances(&mut self) -> &mut Vec<SkinnedObject3DInstance> {
        &mut self.instances
    }
    pub fn find_available_instance(&self) -> Option<usize> {
        self.instances.iter().position(|v| v.busy == false)
    }
    pub fn increase_instances_capability(&mut self, device: &wgpu::Device) {
        let current_capacity = self.instances.capacity();
        let new_capacity = current_capacity * 2;
        println!("new capacity is {new_capacity}");
        self.instances.reserve(new_capacity - self.instances.capacity());
        for _ in 0..current_capacity {
            self.instances.push(SkinnedObject3DInstance::new(self.skeleton.clone(), self.animation_clips.clone()));
        }
        self.instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: (new_capacity * std::mem::size_of::<InstanceRaw>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
    }
    pub fn get_taken_instances_count(&self) -> usize {
        self.instances.iter().filter(|i| i.busy == true).count()
    }
    pub fn get_instance_buffer_slice(&self) -> wgpu::BufferSlice {
        self.instances_buffer.slice(..)
    }
    pub fn update_skeleton(&mut self, queue: &wgpu::Queue) {
        let size = std::mem::size_of::<Mat4x4>();
        for (idx, instance) in self.instances.iter().enumerate() {
            if instance.busy {
                queue.write_buffer(
                    &self.skeletons_buffer,
                    (idx * size * instance.skeleton.bones.len()) as wgpu::BufferAddress,
                    bytemuck::cast_slice(&instance.skeleton.to_raw_transform()),
                );
            }
        }
        queue.write_buffer(
            &self.skeletons_bind_inverse_buffer,
            0,
            bytemuck::cast_slice(&self.model.skeleton.to_raw_inverse_bind_matrices()),
        );
    }
    pub fn set_animations(&mut self, clips: Vec<AnimationClip>) {
        let mut animation_clips = RefCell::borrow_mut(&self.animation_clips);
        animation_clips.clear();
        animation_clips.extend(clips);
    }
    pub fn add_animation(&mut self, clip: AnimationClip) {
        let mut animation_clips = RefCell::borrow_mut(&self.animation_clips);
        if animation_clips.iter().find(|c| c.name == clip.name).is_none() {
            animation_clips.push(clip);
        }
    }
}

#[derive(Debug)]
pub enum Object3D {
    Simple(SimpleObject3D),
    Skinned(SkinnedObject3D)
}

impl Object3D {

    pub fn from_simple_model(device: &wgpu::Device, bind_group_layouts: &SimpleModelBindGroupLayouts, model: SimpleModel) -> Self {
        let mut instances = Vec::new();
        for _ in 0..INITIAL_INSTANCES_COUNT {
            instances.push(SimpleObject3DInstance::new());
        }
        let instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances.iter().map(|i| i.to_instance_raw()).collect::<Vec<_>>()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let instances_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts.instances,
            entries: &[],
            label: None,
        });
        Self::Simple(SimpleObject3D {
            id: generate_unique_string(),
            instances,
            instances_buffer,
            instances_bind_group,
            model,
        })
    }

    pub fn from_skinned_model(device: &wgpu::Device, bind_group_layouts: &SkinnedModelBindGroupLayouts, model: SkinnedModel) -> Self {
        let animation_clips = Rc::new(RefCell::new(model.animations.clone()));
        let skeleton = Rc::new(model.skeleton.clone());
        let mut instances = Vec::new();
        for _ in 0..INITIAL_INSTANCES_COUNT {
            instances.push(SkinnedObject3DInstance::new(skeleton.clone(), animation_clips.clone()));
        }
        let instances_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances.iter().map(|i| i.to_instance_raw()).collect::<Vec<_>>()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let skeletons_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skeleton Buffer"),
            contents: bytemuck::cast_slice(
                &instances
                    .iter()
                    .fold(Vec::<Mat4x4>::new(), |mut acc, i| {
                        acc.extend(i.to_skeleton_raw().iter());
                        acc
                    })
            ),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let skeletons_bind_inverse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skeleton Bind Inverse Buffer"),
            contents: bytemuck::cast_slice(&model.skeleton.to_raw_inverse_bind_matrices()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let skinning_informations_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Skinning Informations Buffer"),
            contents: bytemuck::cast_slice(&[model.skeleton.to_raw_skinning_informations()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let instances_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layouts.instances,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: skeletons_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: skeletons_bind_inverse_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: skinning_informations_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        Self::Skinned(SkinnedObject3D {
            id: generate_unique_string(),
            skeleton,
            animation_clips,
            instances,
            instances_buffer,
            skeletons_buffer,
            skeletons_bind_inverse_buffer,
            instances_bind_group,
            model,
        })
    }
}


#[derive(Clone, Debug)]
pub struct SimpleObject3DInstance {
    pub id: String,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    needs_update: bool,
    busy: bool
}

impl SimpleObject3DInstance {
    pub fn new() -> Self {
        Self {
            id: generate_unique_string(),
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            needs_update: false,
            busy: false
        }
    }
    pub fn update(&mut self, _delta_ms: f64) {
        if !self.busy { return }
    }
    pub fn take(&mut self) {
        self.busy = true;
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.needs_update = true;
    }
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.needs_update = true;
    }
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.needs_update = true;   
    }
    pub fn to_instance_raw(&self) -> InstanceRaw {
        InstanceRaw::new(self.position, self.rotation, self.scale)
    }
}

#[derive(Clone, Debug)]
pub struct SkinnedObject3DInstance {
    pub id: String,
    pub mixer: AnimationMixer,
    skeleton: SkeletonInstance,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    needs_update: bool,
    busy: bool
}

impl SkinnedObject3DInstance {
    pub fn new(skeleton: Rc<Skeleton>, animation_clips: Rc<RefCell<Vec<AnimationClip>>>) -> Self {
        Self {
            id: generate_unique_string(),
            mixer: AnimationMixer::new(animation_clips, false),
            skeleton: skeleton.create_instance(),
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            rotation: cgmath::Quaternion::one(),
            scale: cgmath::Vector3::new(1.0, 1.0, 1.0),
            needs_update: false,
            busy: false
        }
    }
    pub fn update(&mut self, delta_ms: f64) {
        if !self.busy { return }
        self.mixer.update(delta_ms);
        self.mixer.apply_on_skeleton(&mut self.skeleton);
    }
    pub fn take(&mut self) {
        self.busy = true;
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.needs_update = true;
    }
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.needs_update = true;
    }
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.needs_update = true;   
    }
    pub fn to_instance_raw(&self) -> InstanceRaw {
        InstanceRaw::new(self.position, self.rotation, self.scale)
    }
    pub fn to_skeleton_raw(&self) -> Vec<Mat4x4> {
        self.skeleton.to_raw_transform()
    }
}

pub trait TranslateWithScene {
    fn translate(&mut self, x: f32, y: f32, z: f32, scene: &mut Scene);
}

pub trait AdditiveTranslationWithScene {
    fn additive_translation(&mut self, x: f32, y: f32, z: f32, scene: &mut Scene);
}

pub trait RotateWithScene {
    fn rotate(&mut self, w: f32, xi: f32, yj: f32, zk: f32, scene: &mut Scene);
}

pub trait Translate {
    fn translate(&mut self, value: &[f32; 3]);
}
pub trait AdditiveTranslation {
    fn additive_translation(&mut self, value: &[f32; 3]);
}

pub trait Rotate {
    fn rotate(&mut self, value: &[f32; 4]);
}

pub trait Scale {
    fn scale(&mut self, value: &[f32; 3]);
}

impl Translate for SkinnedObject3DInstance {
    fn translate(&mut self, value: &[f32; 3]) {
        self.set_position(cgmath::Vector3 { 
            x: value[0], 
            y: value[1], 
            z: value[2] 
        });
    }
}

impl AdditiveTranslation for SkinnedObject3DInstance {
    fn additive_translation(&mut self, value: &[f32; 3]) {
        let position = self.position.clone() + Vector3::new(value[0], value[1], value[2]);
        self.set_position(position);
    }
}

impl Translate for SimpleObject3DInstance {
    fn translate(&mut self, value: &[f32; 3]) {
        self.set_position(cgmath::Vector3 { 
            x: value[0], 
            y: value[1], 
            z: value[2] 
        });
    }
}

impl Position for SkinnedObject3DInstance {
    fn get_position(&mut self) -> [f32; 3] {
        self.position.into()
    }
}

impl GroundAttachable for SkinnedObject3DInstance {}

impl Rotate for SkinnedObject3DInstance {
    fn rotate(&mut self, value: &[f32; 4]) {
        self.set_rotation(cgmath::Quaternion::new(value[0], value[1], value[2], value[3]));
    }
}

pub trait Position {
    fn get_position(&mut self) -> [f32; 3];
}


pub trait GroundAttachable: Translate + Position {
    fn set_on_the_ground(&mut self, terrain: &Terrain) -> [f32; 3] {
        const GROUND_RAYCAST_OFFSET: f32 = 10.0; // raycast from an higher position
        const DIRECTION: [f32; 3] = [0.0, -1.0, 0.0]; // downside
        let position = self.get_position();
        if let Some(chunk) = terrain.get_chunk_at(&position) {
            let mut origin = position.clone();
            origin[1] += GROUND_RAYCAST_OFFSET;
            let raycaster = Raycaster::new(origin, DIRECTION);
            if let Some(distance) = raycaster.intersects_first(&chunk.terrain_plane.vertices, &chunk.terrain_plane.indices, None) {
                let new_position = [
                    position[0], 
                    position[1] - distance + GROUND_RAYCAST_OFFSET, 
                    position[2]
                ];
                self.translate(&new_position);
                return new_position;
            }
        }
        position
    }
    fn get_distance_to_ground(&mut self, direction: [f32; 3], terrain: &Terrain) -> f32 {
        // TODO: 
        // 1. pass direction in param
        // 2. pass ray length in raycast
        let position = self.get_position();
        if let Some(chunk) = terrain.get_chunk_at(&position) {
            let origin = position.clone();
            let raycaster = Raycaster::new(origin, direction);
            if let Some(distance) = raycaster.intersects_first(&chunk.terrain_plane.vertices, &chunk.terrain_plane.indices, None) {
                return distance
            }
        }
        0.0
    } 
}
