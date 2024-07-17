use super::object_3d::{self, Object3D};
use super::model::DrawModel;

pub struct Scene<'a> {
    objects_3d: Vec<Object3D<'a>>
}

impl Scene<'_> {
    pub fn new() -> Scene<'static> {
        Scene {
            objects_3d: Vec::new()
        }
    }
    pub fn add(&mut self, object_3d: Object3D<'static>) {
        self.objects_3d.push(object_3d)
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
        for object_3d in scene.objects_3d.iter_mut() {
            object_3d.update_instances(queue);
            if let Some(model) = &object_3d.model {
                self.set_vertex_buffer(1, object_3d.get_instance_buffer_slice());
                self.draw_model_instanced(
                    model, 
                    0..object_3d.get_taken_instances_count() as u32, 
                    camera_bind_group, 
                    light_bind_group
                );
            }
        }
    }
}