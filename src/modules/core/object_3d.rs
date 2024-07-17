use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use super::{instance::InstanceRaw, model::Model};

type Mat4 = cgmath::Matrix4<f32>;
type Vec3 = cgmath::Vector3<f32>;
type Quat = cgmath::Quaternion<f32>;

const INITIAL_INSTANCES_COUNT: usize = 100;

pub struct Object3D<'a> {
    pub model: Option<Model>,
    instances: Vec<Object3DInstance>,
    instance_buffer: wgpu::Buffer,
    childrens: Vec<&'a Object3D<'a>>,
    parent: Option<&'a Object3D<'a>>,
}

impl Object3D<'_> {
    pub fn new(device: &wgpu::Device, model: Option<Model>) -> Object3D<'static> {
        let instances = vec![
            Object3DInstance::new();
            INITIAL_INSTANCES_COUNT
        ];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances.iter().map(|i| i.to_instance_raw()).collect::<Vec<_>>()),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Object3D {
            instances,
            instance_buffer,
            childrens: Vec::new(),
            parent: None,
            model,
        }
    }
    pub fn request_instance(&mut self, device: &wgpu::Device) -> &mut Object3DInstance {
        if let Some(index) = self.find_available_instance() {
            return self.instances.get_mut(index).unwrap();
        } 
        println!("oui");
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


#[derive(Clone)]
pub struct Object3DInstance {
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
            rotation: cgmath::Quaternion::new(0.0, 0.0, 0.0, 0.0),
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
    pub fn get_matrix(&self) -> Mat4 {
        self.matrix
    }
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.needs_update = true;
    }
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.needs_update = true;
    }
    pub fn get_world_matrix(&self) -> Mat4 {
        self.matrix_world
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