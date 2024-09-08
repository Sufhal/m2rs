struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

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
    // out.texture_indices = model.texture_indices;
    return out;
}


// Fragment shader
struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1) var<uniform> light: Light;

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

struct ChunkInformations {
    textures_count: u32,
}

@group(1) @binding(1) var<uniform> chunk_informations: ChunkInformations;
@group(1) @binding(2) var sampler_tex: sampler;
@group(1) @binding(3) var tex_0: texture_2d<f32>;
@group(1) @binding(4) var tex_alpha_map_0: texture_2d<f32>;
@group(1) @binding(5) var tex_1: texture_2d<f32>;
@group(1) @binding(6) var tex_alpha_map_1: texture_2d<f32>;
@group(1) @binding(7) var tex_2: texture_2d<f32>;
@group(1) @binding(8) var tex_alpha_map_2: texture_2d<f32>;
@group(1) @binding(9) var tex_3: texture_2d<f32>;
@group(1) @binding(10) var tex_alpha_map_3: texture_2d<f32>;
@group(1) @binding(11) var tex_4: texture_2d<f32>;
@group(1) @binding(12) var tex_alpha_map_4: texture_2d<f32>;
@group(1) @binding(13) var tex_5: texture_2d<f32>;
@group(1) @binding(14) var tex_alpha_map_5: texture_2d<f32>;
@group(1) @binding(15) var tex_6: texture_2d<f32>;
@group(1) @binding(16) var tex_alpha_map_6: texture_2d<f32>;
@group(1) @binding(17) var tex_7: texture_2d<f32>;
@group(1) @binding(18) var tex_alpha_map_7: texture_2d<f32>;
@group(1) @binding(19) var sampler_alpha: sampler;

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn denormalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return value * (max - min) + min;
}

fn ease_in_quart(x: f32) -> f32 {
    return x * x * x * x;
}

fn ease_out_quart(x: f32) -> f32 {
    return 1.0 - pow(1.0 - x, 4.0);
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

    var splat = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    let uv = in.tex_coords;
    let tex_uv = uv * 40.0; // TODO: map level textureset.json contains data about this factor
    let t0 = textureSample(tex_0, sampler_tex, tex_uv) * textureSample(tex_alpha_map_0, sampler_alpha, uv).r;
    let t1 = textureSample(tex_1, sampler_tex, tex_uv) * textureSample(tex_alpha_map_1, sampler_alpha, uv).r;
    let t2 = textureSample(tex_2, sampler_tex, tex_uv) * textureSample(tex_alpha_map_2, sampler_alpha, uv).r;
    let t3 = textureSample(tex_3, sampler_tex, tex_uv) * textureSample(tex_alpha_map_3, sampler_alpha, uv).r;
    let t4 = textureSample(tex_4, sampler_tex, tex_uv) * textureSample(tex_alpha_map_4, sampler_alpha, uv).r;
    let t5 = textureSample(tex_5, sampler_tex, tex_uv) * textureSample(tex_alpha_map_5, sampler_alpha, uv).r;
    let t6 = textureSample(tex_6, sampler_tex, tex_uv) * textureSample(tex_alpha_map_6, sampler_alpha, uv).r;
    let t7 = textureSample(tex_7, sampler_tex, tex_uv) * textureSample(tex_alpha_map_7, sampler_alpha, uv).r;

    for (var i: u32 = 0; i < chunk_informations.textures_count; i = i + 1) {
        switch i {
            case 0u: {
                splat += t0;
            }
            case 1u: {
                splat += t1;
            }
            case 2u: {
                splat += t2;
            }
            case 3u: {
                splat += t3;
            }
            case 4u: {
                splat += t4;
            }
            case 5u: {
                splat += t5;
            }
            case 6u: {
                splat += t6;
            }
            case 7u: {
                splat += t7;
            }
            default: {}
        }
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

    let result = (ambient_color + sun_diffuse_color + moon_diffuse_color + sun.material_emissive.rgb) * splat.xyz;

    return vec4<f32>(result, 1.0);
}
 