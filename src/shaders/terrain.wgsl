struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(2) var<uniform> transform: TransformUniform;

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
    @location(4) color: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    
    var transformed_model_matrix = transform.transform;

    var world_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.position = model.position;
    out.color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    return out;
}


// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1)
var<uniform> light: Light;

fn wireframePattern(uv: vec2<f32>) -> f32 {
    let lineWidth = 0.05; // Width of the wireframe lines
    let uvMod = fract(uv * 30.0); // Adjust 10.0 to control density of the wireframe
    let edge = step(uvMod.x, lineWidth) + step(uvMod.y, lineWidth) + step(1.0 - uvMod.x, lineWidth) + step(1.0 - uvMod.y, lineWidth);
    return clamp(edge, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    // let ambient_strength = 1.0;
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color);

    let wireframeEnabled = false;

    if wireframeEnabled {
        let final_color = vec4<f32>(result, 1.0);
        let wireframe = wireframePattern(in.tex_coords);
        return mix(final_color, vec4<f32>(1.0, 0.0, 0.0, 1.0), wireframe);
    }

    return vec4<f32>(result * in.color.xyz, 1.0);
}
 