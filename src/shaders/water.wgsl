struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct Cycle {
    day_factor: f32,
    night_factor: f32,
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
    day_near: f32,
    day_far: f32,
    day_color: vec4<f32>,
    night_near: f32,
    night_far: f32,
    night_color: vec4<f32>,
}
@group(0) @binding(4) var<uniform> fog: Fog;

struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(0) var<uniform> transform: TransformUniform;
struct Water {
    texture_index: f32,
    time: f32,
    count: u32,
}
@group(1) @binding(1) var<uniform> water: Water;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}
struct WaterDepth {
    @location(3) depth: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) position: vec3<f32>,
    @location(4) depth: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
    water_depth: WaterDepth
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    var transformed_model_matrix = transform.transform;
    let wave_offset = sin(water.time * 1.5) / 20.;
    var position = model.position;
    position.y += wave_offset;
    var world_position: vec4<f32> = transformed_model_matrix * vec4<f32>(position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.position = model.position;
    out.depth = water_depth.depth + wave_offset;
    return out;
}


// Fragment shader
struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1) var<uniform> light: Light;

@group(1) @binding(2) var sampler_tex: sampler;
@group(1) @binding(3) var tex_atlas: texture_2d<f32>;

fn get_uv_in_atlas(uv: vec2<f32>, texture_index: u32, atlas_size: vec2<u32>) -> vec2<f32> {
    let column: f32 = f32(texture_index) % f32(atlas_size.x);
    let line: f32 = floor(f32(texture_index) / f32(atlas_size.y));
    let texture_size: f32 = 1.0 / f32(atlas_size.y);
    let repeated_uv = fract(uv * vec2<f32>(40.0, 40.0));
    return vec2<f32>(
        (column * texture_size) + (repeated_uv.x * texture_size),
        (line * texture_size) + (repeated_uv.y * texture_size),
    );
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
    let min_transparency = .8;
    let max_transparency = .0;
    let opaque_depth_limit = 1.;
    var alpha = min_transparency;

    let current: u32 = u32(floor(water.texture_index));
    var next: u32 = current + 1u;
    if next == water.count {
        next = 0u;
    }

    let water_color: vec3<f32> = mix(
        textureSample(tex_atlas, sampler_tex, get_uv_in_atlas(in.tex_coords, current, vec2<u32>(8, 8))).rgb,
        textureSample(tex_atlas, sampler_tex, get_uv_in_atlas(in.tex_coords, next, vec2<u32>(8, 8))).rgb,
        water.texture_index - f32(current)
    );

    if in.depth < opaque_depth_limit {
        alpha = mix(max_transparency, min_transparency, normalize_value_between(in.depth, 0., opaque_depth_limit));
    }

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

    let result = (ambient_color + sun_diffuse_color + moon_diffuse_color + sun.material_emissive.rgb) * water_color.xyz;

    // fog
    let fog_color = mix(fog.night_color.rgb, fog.day_color.rgb, sun_light_factor);
    let fog_near = mix(fog.night_near, fog.day_near, sun_light_factor);
    let fog_far = mix(fog.night_far, fog.day_far, sun_light_factor);
    let distance_to_camera = length(camera.view_pos.xyz - in.world_position);
    let fog_factor = clamp((distance_to_camera - fog_near) / (fog_far - fog_near), 0.0, 1.0);
    let final_color = mix(result.rgb, fog_color, fog_factor);

    return vec4<f32>(final_color, alpha);
}