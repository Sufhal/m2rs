struct InstanceInput {
    @location(8) model_matrix_0: vec4<f32>,
    @location(9) model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,
    @location(12) normal_matrix_0: vec3<f32>,
    @location(13) normal_matrix_1: vec3<f32>,
    @location(14) normal_matrix_2: vec3<f32>,
};

struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(2) var<uniform> transform: TransformUniform;

struct Light {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) weights: vec4<f32>,
    @location(4) joints: vec4<u32>,
}

struct Mat4x4 {
    data: mat4x4<f32>,
}
@group(2) @binding(0) var<storage, read> bones_matrices: array<Mat4x4>;
@group(2) @binding(1) var<storage, read> bones_inverse_bind_matrices: array<Mat4x4>;

struct SkinningInformations {
    bones_count: u32
}
@group(2) @binding(2) var<uniform> skinning_informations: SkinningInformations;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(instance_index) instance_index: u32,
) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let joint0 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[0]].data * bones_inverse_bind_matrices[model.joints[0]].data;
    let joint1 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[1]].data * bones_inverse_bind_matrices[model.joints[1]].data;
    let joint2 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[2]].data * bones_inverse_bind_matrices[model.joints[2]].data;
    let joint3 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[3]].data * bones_inverse_bind_matrices[model.joints[3]].data;
    let skin_matrix = 
        joint0 * model.weights[0] +
        joint1 * model.weights[1] +
        joint2 * model.weights[2] +
        joint3 * model.weights[3];
    
    var transformed_model_matrix = model_matrix * transform.transform * skin_matrix;
    let model_view_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0); 
    return light.view_proj * model_view_position;
}