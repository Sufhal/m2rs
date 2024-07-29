use cgmath::{One, Quaternion, Rad, Rotation3, SquareMatrix};
use wgpu::util::DeviceExt;

use crate::modules::utils::id_gen::generate_unique_string;

use super::{instance::InstanceRaw, model::Model};

type Mat4 = cgmath::Matrix4<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Quat = cgmath::Quaternion<f32>;

const INITIAL_INSTANCES_COUNT: usize = 100;

#[derive(Debug)]
pub struct Object3D {
    pub id: String,
    pub model: Model,
    instances: Vec<Object3DInstance>,
    instance_buffer: wgpu::Buffer
}

impl Object3D {
    pub fn new(device: &wgpu::Device, model: Model) -> Self {
        let instances = vec![
            Object3DInstance::new();
            INITIAL_INSTANCES_COUNT
        ];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances.iter().map(|i| i.to_instance_raw()).collect::<Vec<_>>()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            id: generate_unique_string(),
            instances,
            instance_buffer,
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
                    &self.instance_buffer,
                    (index * size) as wgpu::BufferAddress,
                    bytemuck::cast_slice(&[instance.to_instance_raw()]),
                );
            }
        }
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
            self.instances.push(Object3DInstance::new());
        }
        self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
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
        self.instance_buffer.slice(..)
    }
}


#[derive(Clone, Debug)]
pub struct Object3DInstance {
    pub id: String,
    rotation: Quat,
    position: Vec3,
    matrix: Mat4,
    matrix_world: Mat4,
    needs_update: bool,
    busy: bool
}

impl Object3DInstance {
    pub fn new() -> Object3DInstance {
        Object3DInstance {
            id: generate_unique_string(),
            rotation: cgmath::Quaternion::one(),
            position: cgmath::Vector3::new(0.0, 0.0, 0.0),
            matrix: cgmath::Matrix4::identity(),
            matrix_world: cgmath::Matrix4::identity(),
            needs_update: false,
            busy: false
        }
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
    pub fn to_instance_raw(&self) -> InstanceRaw {
        InstanceRaw::new(self.position, self.rotation)
    }
    pub fn update(&mut self) {
        if self.needs_update {
            self.needs_update = false;
        }
    }
    pub fn dispose(&mut self) {
        let default = Self::new();
        self.rotation = default.rotation;
        self.position = default.position;
        self.matrix = default.matrix;
        self.matrix_world = default.matrix_world;
        self.busy = false;
        self.needs_update = false;
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
}