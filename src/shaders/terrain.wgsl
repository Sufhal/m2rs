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
@group(0) @binding(1)
var<uniform> light: Light;

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

fn get_texture_at(texture_index: u32, uv: vec2<f32>) -> vec4<f32> {
    let tex_uv = uv * 40.0;
    if texture_index == 0 {
        let alpha = textureSampleLevel(tex_alpha_map_0, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_0, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 1 {
        let alpha = textureSampleLevel(tex_alpha_map_1, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_1, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 2 {
        let alpha = textureSampleLevel(tex_alpha_map_2, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_2, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 3 {
        let alpha = textureSampleLevel(tex_alpha_map_3, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_3, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 4 {
        let alpha = textureSampleLevel(tex_alpha_map_4, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_4, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 5 {
        let alpha = textureSampleLevel(tex_alpha_map_5, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_5, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 6 {
        let alpha = textureSampleLevel(tex_alpha_map_6, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_6, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    else if texture_index == 7 {
        let alpha = textureSampleLevel(tex_alpha_map_7, sampler_tex, uv, 0.0);
        return textureSampleLevel(tex_7, sampler_tex, tex_uv, 0.0) * alpha.r;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;
    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let result = (ambient_color + diffuse_color); // disabled specular, we don't want terrain to reflect light

    // let object_color: vec4<f32> = textureSample(tex_tile, tex_sampler, in.tex_coords);
    // let tile: vec4<f32> = textureSample(tex_tile, sampler_tex, in.tex_coords);
    // let upscaled_tile: f32 = tile.r * 255.0;
    // let tile_texture_index_a: u32 = u32(floor(upscaled_tile));
    // let tile_texture_index_b: u32 = u32(ceil(upscaled_tile));
    // let alpha: f32 = upscaled_tile - floor(upscaled_tile);
    // let uv = in.tex_coords * 40.;

    // splat += get_texture_at(tile_texture_index_a, uv) * alpha;
    // splat += get_texture_at(tile_texture_index_b, uv) * (1 - alpha);

    var splat = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    for (var i: u32 = 0; i < chunk_informations.textures_count; i = i + 1) {
        splat += get_texture_at(i, in.tex_coords);
    }
    return splat;
    // let alpha: vec4<f32> = textureSample(tex_alpha_map_0, sampler_tex, in.tex_coords);
    // return vec4<f32>(splat.xyz, 1.0);
    // return vec4<f32>(alpha.r, 0.0, 0.0, 1.0);
    // return debug;
    // return vec4<f32>(object_color.xyz, 1.0);
}
 