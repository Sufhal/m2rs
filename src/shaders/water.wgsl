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

fn normalize_value_between(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn get_uv_in_atlas(uv: vec2<f32>, texture_index: u32, atlas_size: vec2<u32>) -> vec2<f32> {
    let line = u32(floor(f32(texture_index) / f32(atlas_size.y)));
    let column = u32(f32(texture_index) % f32(atlas_size.x));
    let texture_size = 1.0 / f32(atlas_size.y);

    // Taille de chaque sous-texture dans l'atlas
    let texture_size = vec2<f32>(1.0) / vec2<f32>(atlas_size);

    // Position de la texture indexée dans l'atlas
    let texel_index = vec2<u32>(texture_index % atlas_size.x, texture_index / atlas_size.x);
    let base_uv = vec2<f32>(texel_index) * texture_size;

    // UV dans la sous-région correspondante
    return base_uv + uv * texture_size;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = .1;
    let ambient_color = light.color * ambient_strength;
    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let result = (ambient_color + diffuse_color); // disabled specular, we don't want terrain to reflect light

    let uv = vec2<f32>(in.tex_coords.y, in.tex_coords.x) * 10.0;
    let min_transparency = .8;
    let max_transparency = .0;
    let opaque_depth_limit = 1.;
    var alpha = min_transparency;

    // let water_color = mix(
    //     textureSample(tex_0, sampler_tex, uv),
    //     textureSample(tex_1, sampler_tex, uv),
    //     water.factor
    // );

    let current: u32 = u32(floor(water.texture_index));
    var next: u32 = current + 1u;
    if current != water.count {
        next = u32(ceil(water.texture_index));
    }
    // if current as usize == TEXTURES_COUNT - 1 { 0.0 } else { f32::ceil(texture_index) };

    // let water_color = mix(
    //     textureSample(tex_atlas, sampler_tex, get_uv_in_atlas(in.tex_coords, current, vec2<u32>(8, 8))),
    //     textureSample(tex_atlas, sampler_tex, get_uv_in_atlas(in.tex_coords, next, vec2<u32>(8, 8))),
    //     water.texture_index - f32(current)
    // );

    var water_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if current < 30 {
    } else {
        water_color.r = 1.0;
    }

    // let water_color = textureSample(tex_atlas, sampler_tex, get_uv_in_atlas(in.tex_coords, current, vec2<u32>(8, 8)));

    if in.depth < opaque_depth_limit {
        alpha = mix(max_transparency, min_transparency, normalize_value_between(in.depth, 0., opaque_depth_limit));
    }
    // if (waterDepth < opaqueDepthLimit)
	// 	transparency = mix(maxTransparency, minTransparency, normalizeValueBetween(waterDepth, 0., opaqueDepthLimit));

    return vec4<f32>(water_color.rgb, alpha);
}
 