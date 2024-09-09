struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct Cycle {
    day_factor: f32,
    night_factor: f32,
}
@group(0) @binding(2) var<uniform> cycle: Cycle;



struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> transform: TransformUniform;

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
}


@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    
    var transformed_model_matrix = transform.transform;

    out.position = model.position;

    var transformed_model_view_matrix = camera.view_matrix * transformed_model_matrix;

    let model_view_position: vec4<f32> = transformed_model_view_matrix * vec4<f32>(model.position, 1.0); 
    out.clip_position = camera.projection_matrix *  model_view_position;

    return out;
}

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn ease_out_expo(x: f32) -> f32 {
    if x == 1.0 {
        return 1.0;
    } else {
        return 1.0 - pow(2.0, -10.0 * x);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}