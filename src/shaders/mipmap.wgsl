// // Vertex Shader
// @vertex
// fn vs_main(
//     @builtin(vertex_index) vertex_index: u32
// ) -> @builtin(position) vec4<f32> {
//     var pos = array<vec2<f32>, 3>(
//         vec2<f32>(-1.0, -1.0),
//         vec2<f32>( 3.0, -1.0),
//         vec2<f32>(-1.0,  3.0)
//     );
//     return vec4<f32>(pos[vertex_index], 0.0, 1.0);
// }

// // Fragment Shader
// @group(0) @binding(0)
// var u_sampler: sampler;
// @group(0) @binding(1)
// var u_texture: texture_2d<f32>;

// @fragment
// fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
//     let tex_coords = frag_coord.xy / vec2<f32>(textureDimensions(u_texture));
//     return textureSample(u_texture, u_sampler, tex_coords  2.0);
// }

struct VSOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
};

@vertex 
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> VSOutput {
    var pos = array<vec2<f32>, 6>(
        vec2<f32>( 0.0,  0.0),  // center
        vec2<f32>( 1.0,  0.0),  // right, center
        vec2<f32>( 0.0,  1.0),  // center, top

        // 2st triangle
        vec2<f32>( 0.0,  1.0),  // center, top
        vec2<f32>( 1.0,  0.0),  // right, center
        vec2<f32>( 1.0,  1.0),  // right, top
    );

    var vsOutput: VSOutput;
    let xy = pos[vertex_index];
    vsOutput.position = vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
    vsOutput.texcoord = vec2<f32>(xy.x, 1.0 - xy.y);
    return vsOutput;
}

@group(0) @binding(0) var ourSampler: sampler;
@group(0) @binding(1) var ourTexture: texture_2d<f32>;

@fragment 
fn fs_main(fsInput: VSOutput) -> @location(0) vec4f {
    return textureSample(ourTexture, ourSampler, fsInput.texcoord);
}