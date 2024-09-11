// Vertex shader

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

struct Cycle {
    day_factor: f32,
    night_factor: f32,
    padding: vec2<f32>,
}
@group(0) @binding(2) var<uniform> cycle: Cycle;

struct Sun {
    sun_position: vec4<f32>,
    moon_position: vec4<f32>,
    day_material_diffuse: vec4<f32>,
    day_material_ambient: vec4<f32>,
    day_material_emissive: vec4<f32>,
    day_background_diffuse: vec4<f32>,
    day_background_ambient: vec4<f32>,
    day_character_diffuse: vec4<f32>,
    day_character_ambient: vec4<f32>,
    night_material_diffuse: vec4<f32>,
    night_material_ambient: vec4<f32>,
    night_material_emissive: vec4<f32>,
    night_background_diffuse: vec4<f32>,
    night_background_ambient: vec4<f32>,
    night_character_diffuse: vec4<f32>,
    night_character_ambient: vec4<f32>,
}
@group(0) @binding(3) var<uniform> sun: Sun;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) position: vec3<f32>,
    @location(4) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
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
    var world_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0);
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = normal_matrix * model.normal;
    out.world_position = world_position.xyz;
    out.position = model.position;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    var transformed_model_view_matrix = camera.view_matrix * transformed_model_matrix;
    let model_view_position: vec4<f32> = transformed_model_view_matrix * vec4<f32>(model.position, 1.0); 
    out.clip_position = camera.projection_matrix *  model_view_position;
    return out;
}