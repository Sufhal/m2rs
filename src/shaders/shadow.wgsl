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
    view_proj: mat4x4<f32>
}
@group(0) @binding(0) var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    var transformed_model_matrix = model_matrix * transform.transform;
    let model_view_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0); 
    return light.view_proj * model_view_position;
}