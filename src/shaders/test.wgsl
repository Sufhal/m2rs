
struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> transform: TransformUniform;
@group(0) @binding(1) var shadow_sampler: sampler_comparison;
@group(0) @binding(2) var shadow_texture: texture_depth_2d;
struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
};
@group(0) @binding(3) var<uniform> camera: CameraUniform;

struct Light {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
}
@group(0) @binding(4) var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) position: vec3<f32>,
    @location(4) shadow_coords: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    var transformed_model_matrix = transform.transform;
    var world_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.position = model.position;
    let light_space_position = light.view_proj * transformed_model_matrix * vec4<f32>(model.position, 1.0);
    out.shadow_coords = light_space_position.xyz / light_space_position.w;
    return out;
}




@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let light_space_position = light.view_proj * vec4<f32>(in.world_position, 1.0);
    // var proj_coords = light_space_position.xyz / light_space_position.w;
    // proj_coords = proj_coords * 0.5 + 0.5;

    let light_space_position = light.view_proj * vec4<f32>(in.world_position, 1.0);
    let proj_correction = 1.0 / light_space_position.w;
    let proj_coords = light_space_position.xy * vec2<f32>(0.5, -0.5) * proj_correction + 0.5;


    let shadow = textureSampleCompare(
        shadow_texture,
        shadow_sampler,
        proj_coords.xy,
        light_space_position.z * proj_correction,
    );
    let color = vec3<f32>(1.0, 1.0, 0.0) * shadow;
    let shadow_color = vec3<f32>(1.0, 0.0, 1.0) * (1.0 - shadow);

    return vec4<f32>(color + shadow_color, 1.0);
}