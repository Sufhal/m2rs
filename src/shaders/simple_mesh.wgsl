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

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
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


// Fragment shader

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct Light {
    position: vec3<f32>,
    _padding1: f32,
    color: vec3<f32>,
    _padding2: f32,
}
@group(0) @binding(1) var<uniform> light: Light;

struct Cycle {
    day_factor: f32,
    night_factor: f32,
    padding: vec2<f32>,
}
@group(0) @binding(2) var<uniform> cycle: Cycle;

struct Sun {
    sun_position: vec4<f32>,
    moon_position: vec4<f32>,
    material_diffuse: vec4<f32>,
    material_ambient: vec4<f32>,
    material_emissive: vec4<f32>,
    background_diffuse: vec4<f32>,
    background_ambient: vec4<f32>,
    character_diffuse: vec4<f32>,
    character_ambient: vec4<f32>,
}
@group(0) @binding(3) var<uniform> sun: Sun;

struct Fog {
    day_color: vec4<f32>,
    day_near: f32,
    day_far: f32,
    padding1: f32,
    padding2: f32,
    night_color: vec4<f32>,
    night_near: f32,
    night_far: f32,
    padding3: f32,
    padding4: f32,
}
@group(0) @binding(4) var<uniform> fog: Fog;

fn wireframePattern(uv: vec2<f32>) -> f32 {
    let lineWidth = 0.05; // Width of the wireframe lines
    let uvMod = fract(uv * 30.0); // Adjust 10.0 to control density of the wireframe
    let edge = step(uvMod.x, lineWidth) + step(uvMod.y, lineWidth) + step(1.0 - uvMod.x, lineWidth) + step(1.0 - uvMod.y, lineWidth);
    return clamp(edge, 0.0, 1.0);
}

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn denormalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return value * (max - min) + min;
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
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var sun_light_factor: f32 = 0.0;

    if cycle.day_factor > 0.0 && cycle.day_factor <= 0.5  {
        sun_light_factor = ease_out_expo(normalize_value_between(cycle.day_factor, 0.0, 0.5));
    }
    else if cycle.day_factor > 0.0 && cycle.day_factor <= 1.0 {
        sun_light_factor = ease_out_expo(normalize_value_between(1.0 - cycle.day_factor, 0.0, 0.5));
    }

    let ambient_strength = 0.3;
    let ambient_color = sun.material_ambient.rgb * ambient_strength;

    let sun_light_dir = normalize(sun.sun_position.xyz - in.world_position);
    let sun_diffuse_strength = max(dot(in.world_normal, sun_light_dir), 0.0);
    let sun_diffuse_color = sun.material_diffuse.rgb * sun.background_diffuse.rgb * sun_diffuse_strength * sun_light_factor;

    let moon_light_dir = normalize(sun.moon_position.xyz - in.world_position);
    let moon_diffuse_strength = max(dot(in.world_normal, moon_light_dir), 0.0);
    let moon_diffuse_color = sun.material_diffuse.rgb * sun.background_diffuse.rgb * moon_diffuse_strength * 0.2;

    // let result = (ambient_color + sun_diffuse_color + sun.material_emissive.rgb) * vec3<f32>(1.0, 1.0, 1.0);
    let result = (ambient_color + sun_diffuse_color + moon_diffuse_color + sun.material_emissive.rgb) * object_color.xyz;

    // fog
    let fog_color = mix(fog.night_color.rgb, fog.day_color.rgb, sun_light_factor);
    let fog_near = mix(fog.night_near, fog.day_near, sun_light_factor);
    let fog_far = mix(fog.night_far, fog.day_far, sun_light_factor);
    let distance_to_camera = length(camera.view_pos.xyz - in.world_position);
    let fog_factor = clamp((distance_to_camera - fog_near) / (fog_far - fog_near), 0.0, 1.0);
    let final_color = mix(result.rgb, fog_color, fog_factor);

    return vec4<f32>(final_color, object_color.a);
}
 