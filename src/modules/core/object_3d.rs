use std::{borrow::BorrowMut, cell::{RefCell, RefMut}, rc::Rc};

use cgmath::{One, Quaternion, Rad, Rotation3, SquareMatrix};
use wgpu::util::DeviceExt;

use crate::modules::{pipelines::render_pipeline::{self, RenderBindGroupLayouts, RenderPipeline}, utils::id_gen::generate_unique_string};

use super::{instance::InstanceRaw, model::Model, skinning::{AnimationClip, AnimationMixer, Mat4x4, Skeleton, SkeletonInstance}};

type Mat4 = cgmath::Matrix4<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Quat = cgmath::Quaternion<f32>;

const INITIAL_INSTANCES_COUNT: usize = 100;

#[derive(Debug)]
pub struct Object3D {
    pub id: String,
    pub model: Model,
    pub instances_bind_group: wgpu::BindGroup,
    animation_clips: Rc<RefCell<Vec<AnimationClip>>>,
    skeleton: Rc<Skeleton>,
    instances: Vec<Object3DInstance>,
    instances_buffer: wgpu::Buffer,
    // skeletons: Vec<SkeletonInstance>,
    // skeletons_bind_group: Option<wgpu::BindGroup>,
    skeletons_buffer: wgpu::Buffer,
    skeletons_bind_inverse_buffer: wgpu::Buffer,
    // skeletons_bind_inverse_buffer: wgpu::Buffer,
}

impl Object3D {
    pub fn new(device: &wgpu::Device, bind_group_layouts: &RenderBindGroupLayouts, model: Model) -> Self {
        let animation_clips = Rc::new(RefCell::new(model.animations.clone()));
        let skeleton = Rc::new(model.skeleton.clone());
        let instances = vec![
            Object3DInstance::new(skeleton.clone(), animation_clips.clone());
            INITIAL_INSTANCES_COUNT
        ];
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
            contents: bytemuck::cast_slice(&model.skeleton.to_raw()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        // let skeletons_bind_inverse_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Skeleton Buffer"),
        //     contents: bytemuck::cast_slice(&
        //         if let Some(skeleton) = &model.skeleton {
        //             skeleton.to_raw()
        //         } else {
        //             Vec::new()
        //         }
        //     ),
        //     usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        // });

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
                }
            ],
            label: None,
        });

        Self {
            id: generate_unique_string(),
            skeleton,
            animation_clips,
            instances,
            instances_buffer,
            skeletons_buffer,
            skeletons_bind_inverse_buffer,
            instances_bind_group,
            model,
        }
    }

    pub fn request_instance(&mut self, device: &wgpu::Device) -> &mut Object3DInstance {
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
            bytemuck::cast_slice(&self.model.skeleton.to_raw()),
        );
    }
    pub fn set_animations(&mut self, clips: Vec<AnimationClip>) {
        let mut animation_clips = RefCell::borrow_mut(&self.animation_clips);
        animation_clips.clear();
        animation_clips.extend(clips);
    }
    pub fn get_instance(&mut self, id: &str) -> Option<&mut Object3DInstance> {
        self.instances.iter_mut().find(|i| &i.id == id)
    }
    pub fn get_instances(&mut self) -> &mut Vec<Object3DInstance> {
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
            self.instances.push(Object3DInstance::new(self.skeleton.clone(), self.animation_clips.clone()));
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


#[derive(Clone, Debug)]
pub struct Object3DInstance {
    pub id: String,
    mixer: AnimationMixer,
    skeleton: SkeletonInstance,
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
    needs_update: bool,
    busy: bool
}

impl Object3DInstance {
    pub fn new(skeleton: Rc<Skeleton>, animation_clips: Rc<RefCell<Vec<AnimationClip>>>) -> Object3DInstance {
        Object3DInstance {
            id: generate_unique_string(),
            mixer: AnimationMixer::new(animation_clips, true),
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

pub trait Transform {
    fn add_x_rotation(&mut self, angle: f32);
    fn add_y_rotation(&mut self, angle: f32);
    fn add_z_rotation(&mut self, angle: f32);
    fn add_xyz_rotation(&mut self, x: f32, y: f32, z: f32);
    fn add_x_position(&mut self, value: f32);
    fn add_y_position(&mut self, value: f32);
    fn add_z_position(&mut self, value: f32);
    fn add_xyz_position(&mut self, x: f32, y: f32, z: f32);
    fn add_x_scale(&mut self, value: f32);
    fn add_y_scale(&mut self, value: f32);
    fn add_z_scale(&mut self, value: f32);
    fn add_xyz_scale(&mut self, x: f32, y: f32, z: f32);
}

impl Transform for Object3DInstance {
    fn add_x_rotation(&mut self, angle: f32) {
        let incremental_rotation = Quaternion::from_angle_x(Rad(angle));
        self.set_rotation(self.rotation * incremental_rotation);
    }
    fn add_y_rotation(&mut self, angle: f32) {
        let incremental_rotation = Quaternion::from_angle_y(Rad(angle));
        self.set_rotation(self.rotation * incremental_rotation);
    }
    fn add_z_rotation(&mut self, angle: f32) {
        let incremental_rotation = Quaternion::from_angle_z(Rad(angle));
        self.set_rotation(self.rotation * incremental_rotation);
    }
    fn add_xyz_rotation(&mut self, x: f32, y: f32, z: f32) {
        let incremental_rotation = 
            Quaternion::from_angle_x(Rad(x)) *
            Quaternion::from_angle_y(Rad(y)) *
            Quaternion::from_angle_z(Rad(z));
        self.set_rotation(self.rotation * incremental_rotation);
    }
    fn add_x_position(&mut self, value: f32) {
        self.set_position(self.position + cgmath::Vector3 { x: value, y: 0.0, z: 0.0 });
    }
    fn add_y_position(&mut self, value: f32) {
        self.set_position(self.position + cgmath::Vector3 { x: 0.0, y: value, z: 0.0 });
    }
    fn add_z_position(&mut self, value: f32) {
        self.set_position(self.position + cgmath::Vector3 { x: 0.0, y: 0.0, z: value });
    }
    fn add_xyz_position(&mut self, x: f32, y: f32, z: f32) {
        self.set_position(self.position + cgmath::Vector3 { x, y, z });
    }
    fn add_x_scale(&mut self, value: f32) {
        self.set_scale(self.scale + cgmath::Vector3 { x: value, y: 0.0, z: 0.0 });
    }
    fn add_y_scale(&mut self, value: f32) {
        self.set_scale(self.scale + cgmath::Vector3 { x: 0.0, y: value, z: 0.0 });
    }
    fn add_z_scale(&mut self, value: f32) {
        self.set_scale(self.scale + cgmath::Vector3 { x: 0.0, y: 0.0, z: value });
    }
    fn add_xyz_scale(&mut self, x: f32, y: f32, z: f32) {
        self.set_scale(self.scale + cgmath::Vector3 { x, y, z });
    }
}