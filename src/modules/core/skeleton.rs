use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use super::object::Object;

// get all bones ids
// compute all bones inverse

// create storage buffer to use it like that https://github.com/webgpu/webgpu-samples/blob/main/sample/skinnedMesh/gltf.wgsl

pub struct Skeleton {
	bones_ids: Vec<String>,
	bones_uniforms: Vec<BoneUniform>,
	buffer: wgpu::Buffer,
	bind_group: wgpu::BindGroup
}

impl Skeleton {

	pub fn new(
		device: &wgpu::Device,
		bones_bind_group_layout: &wgpu::BindGroupLayout,
		bones: Vec<&Object>
	) -> Self {
		let mut bones_ids = Vec::with_capacity(bones.len());
		let mut bones_uniforms = Vec::with_capacity(bones.len());
		for bone in bones {
			bones_ids.push(bone.id.clone());
			bones_uniforms.push(
				BoneUniform::new(
					bone.matrix_world.clone(), 
					if let Some(bind_inverse) = cgmath::Matrix4::from(bone.matrix_world).invert() {
						bind_inverse.into()
					} else {
						cgmath::Matrix4::identity().into()
					}
				)
			)
		}
		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Bones Buffer"),
			contents: bytemuck::cast_slice(&bones_uniforms),
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
		});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bones_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: buffer.as_entire_binding(),
				}
			],
			label: Some("bones_bind_group"),
		});
		Self {
			bones_ids,
			bones_uniforms,
			buffer,
			bind_group
		}
	}

	// pub fn set_bones(&mut self, bones: Vec<&Object>) {
	// 	self.bones_ids.clear();
	// 	self.bones_matrices.clear();
	// 	self.bones_bind_inverse_matrices.clear();
	// 	for bone in bones {
	// 		self.bones_ids.push(bone.id.clone());
	// 		self.bones_matrices.push(bone.matrix_world.clone());
	// 		if let Some(bind_inverse) = cgmath::Matrix4::from(bone.matrix_world).invert() {
	// 			self.bones_bind_inverse_matrices.push(bind_inverse.into());
	// 		} else {
	// 			self.bones_bind_inverse_matrices.push(cgmath::Matrix4::identity().into());
	// 		}
	// 	}
	// }

}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct BoneUniform {
	bone_matrix: [[f32; 4]; 4],
	bone_inverse_bind_matrix: [[f32; 4]; 4]
}

impl BoneUniform {
	fn new(bone_matrix: [[f32; 4]; 4], bone_inverse_bind_matrix: [[f32; 4]; 4]) -> Self {
		Self { bone_matrix, bone_inverse_bind_matrix }
	}
	fn update(&mut self, bone_matrix: [[f32; 4]; 4]) {
		self.bone_matrix = bone_matrix;
	}
}