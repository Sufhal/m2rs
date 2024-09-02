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
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct TransformUniform {
    transform: mat4x4<f32>,
};
@group(1) @binding(2) var<uniform> transform: TransformUniform;

struct Mat4x4 {
    data: mat4x4<f32>,
}
@group(2) @binding(0) var<storage, read> bones_matrices: array<Mat4x4>;
@group(2) @binding(1) var<storage, read> bones_inverse_bind_matrices: array<Mat4x4>;

struct SkinningInformations {
    bones_count: u32
}
@group(2) @binding(2) var<uniform> skinning_informations: SkinningInformations;


struct VertexInput {
    // @location(0) position: vec3<f32>,
    // @location(1) _pad1: f32,
    // @location(2) tex_coords: vec2<f32>,
    // @location(3) _pad2: vec2<f32>,
    // @location(4) normal: vec3<f32>,
    // @location(5) _pad3: f32,
    // @location(6) weights: vec4<f32>,
    // @location(7) joints: vec4<u32>,
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) weights: vec4<f32>,
    @location(4) joints: vec4<u32>,
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
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = normal_matrix * model.normal;

    let joint0 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[0]].data * bones_inverse_bind_matrices[model.joints[0]].data;
    let joint1 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[1]].data * bones_inverse_bind_matrices[model.joints[1]].data;
    let joint2 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[2]].data * bones_inverse_bind_matrices[model.joints[2]].data;
    let joint3 = bones_matrices[(instance_index * skinning_informations.bones_count) + model.joints[3]].data * bones_inverse_bind_matrices[model.joints[3]].data;
    let skin_matrix = 
        joint0 * model.weights[0] +
        joint1 * model.weights[1] +
        joint2 * model.weights[2] +
        joint3 * model.weights[3];
    
    var transformed_model_matrix = model_matrix * transform.transform * skin_matrix;

    var world_position: vec4<f32> = transformed_model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.position = model.position;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
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
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // We don't need (or want) much ambient light, so 0.1 is fine
    let ambient_strength = 1.0;
    // let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;

    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular_color = specular_strength * light.color;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    let wireframeEnabled = false;

    if wireframeEnabled {
        let final_color = vec4<f32>(result, object_color.a);
        let wireframe = wireframePattern(in.tex_coords);
        return mix(final_color, vec4<f32>(1.0, 0.0, 0.0, 1.0), wireframe);
    }

    return vec4<f32>(result * in.color.xyz, object_color.a);
}
 