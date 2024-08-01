use super::object::Object;

pub struct Skeleton {
	pub bones_ids: Vec<String>,
	pub bones_matrices: Vec<[[f32; 4]; 4]>,
	pub bones_bind_inverse_matrices: Vec<[[f32; 4]; 4]>,

	// get all bones ids
	// compute all bones inverse

	// create storage buffer to use it like that https://github.com/webgpu/webgpu-samples/blob/main/sample/skinnedMesh/gltf.wgsl
}

impl Skeleton {

	pub fn new() -> Self {
		Self {
			bones_ids: Vec::new(),
			bones_matrices: Vec::new(),
			bones_bind_inverse_matrices: Vec::new(),
		}
	}

	pub fn set_bones(bones: Vec<&Object>) {

	}

	
}